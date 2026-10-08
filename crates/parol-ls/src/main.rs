use std::collections::HashSet;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::{error::Error, result::Result};

use crate::handler::RequestHandler;
use crate::{arguments::Arguments, config::Config};

pub mod arguments;
mod config;
mod convert_to_rng;
pub mod diagnostics;
pub mod document_state;
mod document_symbol;
pub mod errors;
mod formatting;
mod handler;
pub mod parol_ls_grammar;
mod parol_ls_grammar_trait;
mod parol_ls_parser;
mod rng;
mod server;
mod symbol_def;
mod utils;

extern crate clap;
extern crate parol_runtime;

use clap::Parser;

use errors::ServerError;
use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use lsp_types::notification::DidChangeConfiguration;
use lsp_types::request::RegisterCapability;
use lsp_types::{
    CancelParams, CodeActionProviderCapability, HoverProviderCapability, InitializeParams,
    LogMessageParams, MessageType, NumberOrString, OneOf, RenameOptions, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    WorkDoneProgressOptions,
    notification::{
        Cancel, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Exit,
        LogMessage, Notification,
    },
    request::{
        CodeActionRequest, DocumentSymbolRequest, Formatting, GotoDefinition, HoverRequest,
        PrepareRenameRequest, Rename,
    },
};
use lsp_types::{DidOpenTextDocumentParams, Registration, RegistrationParams};
use parol_runtime::log::debug;
use serde::Serialize;
use server::Server;

static GLOBAL_REQUEST_ID: RequestCounter = RequestCounter::new();

struct RequestCounter(AtomicI32);
impl RequestCounter {
    const fn new() -> RequestCounter {
        Self(AtomicI32::new(1000))
    }

    fn next() -> RequestId {
        let id = GLOBAL_REQUEST_ID.0.fetch_add(1, Ordering::SeqCst);
        RequestId::from(id)
    }
}

// Note that this function only sends the request. The response handling is done in the main loop!
pub(crate) fn send_request<R>(conn: Arc<Connection>, params: R::Params) -> Result<(), ServerError>
where
    R: lsp_types::request::Request,
    R::Params: Serialize,
{
    let r = Request::new(RequestCounter::next(), R::METHOD.to_string(), params);
    conn.sender
        .send(r.into())
        .map_err(|err| ServerError::ProtocolError { err: Box::new(err) })
}

fn send_log_message(connection: &Connection, message: impl Into<String>) {
    let params = LogMessageParams {
        typ: MessageType::INFO,
        message: message.into(),
    };
    let params = match serde_json::to_value(params) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to serialize logMessage params: {err}");
            return;
        }
    };
    let method = LogMessage::METHOD.to_string();
    let _ = connection
        .sender
        .send(Message::Notification(lsp_server::Notification { method, params }));
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    debug!("env logger started");

    let args = Arguments::parse();
    eprintln!("Starting parol language server");

    let (connection, io_threads) = if args.stdio {
        Connection::stdio()
    } else {
        Connection::connect((args.ip_address, args.port_number))?
    };
    let connection = Arc::new(connection);

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                will_save: None,
                will_save_wait_until: None,
                save: None,
            },
        )),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        document_formatting_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        ..Default::default()
    })
    .unwrap();

    let initialization_params: InitializeParams =
        serde_json::from_value(connection.initialize(server_capabilities)?)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
    send_log_message(
        &connection,
        format!(
            "parol-ls started: pid={}, exe={}",
            std::process::id(),
            std::env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "<unknown>".to_string())
        ),
    );
    let config = Config::new(initialization_params, args);

    main_loop(connection, config)?;
    io_threads.join()?;

    Ok(())
}

fn main_loop(connection: Arc<Connection>, config: Config) -> Result<(), Box<dyn Error>> {
    eprintln!(
        "Initialization params {:#?}",
        config.initialization_params()
    );
    eprintln!(
        "Initialization options {:#?}",
        config.initialization_options()
    );
    // First initialize the server with the lookahead from the server invocation
    let server = Arc::new(Mutex::new(server::Server::new(
        config.lookahead(),
        config.supports_hierarchical_document_symbols(),
    )));
    // Then update properties from client configuration (i.e. settings).
    server
        .lock()
        .expect("server mutex poisoned")
        .update_configuration(config.config_properties())?;

    if config.supports_dynamic_registration_for_change_config() {
        send_request::<RegisterCapability>(
            connection.clone(),
            RegistrationParams {
                registrations: vec![Registration {
                    id: "workspace/didChangeConfiguration".to_string(),
                    method: "workspace/didChangeConfiguration".to_string(),
                    register_options: None,
                }],
            },
        )?;
    }

    let active_requests = Arc::new(Mutex::new(HashSet::<RequestId>::new()));
    let cancelled_requests = Arc::new(Mutex::new(HashSet::<RequestId>::new()));

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                eprintln!("got request: {req:?}");
                if connection.handle_shutdown(&req)? {
                    // Ensure prompt shutdown even when detached worker threads are still running.
                    eprintln!(
                        "shutdown handled, terminating process pid={}",
                        std::process::id()
                    );
                    send_log_message(
                        &connection,
                        format!("parol-ls shutdown handled: pid={}", std::process::id()),
                    );
                    std::process::exit(0);
                }

                let request_id = req.id.clone();
                {
                    let mut active = active_requests
                        .lock()
                        .expect("active_requests mutex poisoned");
                    active.insert(request_id.clone());
                }
                if take_cancelled_request(&cancelled_requests, &request_id) {
                    let _ = remove_active_request(&active_requests, &request_id);
                    send_request_cancelled(&connection, request_id)?;
                    continue;
                }

                let server = server.clone();
                let connection = connection.clone();
                let active_requests = active_requests.clone();
                let cancelled_requests = cancelled_requests.clone();
                std::thread::spawn(move || {
                    if take_cancelled_request(&cancelled_requests, &request_id) {
                        return;
                    }

                    let response = handle_request(req, server, connection.clone());

                    if remove_active_request(&active_requests, &request_id)
                        && let Some(response) = response
                    {
                        eprintln!("send response: {:?}", response);
                        if let Err(err) = connection.sender.send(Message::Response(response)) {
                            eprintln!("failed to send response for request #{request_id}: {err}");
                        }
                    }
                });
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(not) => {
                process_notification(
                    not,
                    connection.clone(),
                    &server,
                    &active_requests,
                    &cancelled_requests,
                )?;
            }
        }
    }
    Ok(())
}

fn process_notification(
    not: lsp_server::Notification,
    connection: Arc<Connection>,
    server: &Arc<Mutex<Server>>,
    active_requests: &Arc<Mutex<HashSet<RequestId>>>,
    cancelled_requests: &Arc<Mutex<HashSet<RequestId>>>,
) -> Result<(), Box<dyn Error>> {
    eprintln!("got notification: {not:?}");
    match not.method.as_str() {
        Cancel::METHOD => {
            let params: CancelParams = not.extract(Cancel::METHOD)?;
            if let Some(id) = to_request_id(params.id) {
                {
                    let mut cancelled = cancelled_requests
                        .lock()
                        .expect("cancelled_requests mutex poisoned");
                    cancelled.insert(id.clone());
                }
                if remove_active_request(active_requests, &id) {
                    send_request_cancelled(&connection, id)?;
                }
            }
        }
        DidOpenTextDocument::METHOD => {
            let params: DidOpenTextDocumentParams = not.extract(DidOpenTextDocument::METHOD)?;
            let uri = params.text_document.uri.clone();
            let version = params.text_document.version;
            let text = params.text_document.text;
            server
                .lock()
                .expect("server mutex poisoned")
                .handle_open_document(connection, uri, version, text)?
        }
        DidChangeTextDocument::METHOD => server
            .lock()
            .expect("server mutex poisoned")
            .handle_change_document(connection, not)?,
        DidCloseTextDocument::METHOD => server
            .lock()
            .expect("server mutex poisoned")
            .handle_close_document(not)?,
        DidChangeConfiguration::METHOD => server
            .lock()
            .expect("server mutex poisoned")
            .handle_changed_configuration(not)?,
        Exit::METHOD => {
            eprintln!(
                "exit notification received, terminating pid={}",
                std::process::id()
            );
            send_log_message(
                &connection,
                format!("parol-ls exit notification: pid={}", std::process::id()),
            );
            std::process::exit(0);
        }
        _ => {}
    }
    Ok(())
}

fn handle_request(
    req: Request,
    server: Arc<Mutex<Server>>,
    connection: Arc<Connection>,
) -> Option<Response> {
    match req.method.as_str() {
        <GotoDefinition as lsp_types::request::Request>::METHOD => {
            request_response::<GotoDefinition>(req, server, connection)
        }
        <HoverRequest as lsp_types::request::Request>::METHOD => {
            request_response::<HoverRequest>(req, server, connection)
        }
        <DocumentSymbolRequest as lsp_types::request::Request>::METHOD => {
            request_response::<DocumentSymbolRequest>(req, server, connection)
        }
        <PrepareRenameRequest as lsp_types::request::Request>::METHOD => {
            request_response::<PrepareRenameRequest>(req, server, connection)
        }
        <Rename as lsp_types::request::Request>::METHOD => {
            request_response::<Rename>(req, server, connection)
        }
        <Formatting as lsp_types::request::Request>::METHOD => {
            request_response::<Formatting>(req, server, connection)
        }
        <CodeActionRequest as lsp_types::request::Request>::METHOD => {
            request_response::<CodeActionRequest>(req, server, connection)
        }
        _ => {
            eprintln!("Unhandled request {}", req.method);
            None
        }
    }
}

fn request_response<R>(
    req: Request,
    server: Arc<Mutex<Server>>,
    connection: Arc<Connection>,
) -> Option<Response>
where
    R: lsp_types::request::Request + RequestHandler,
    R::Params: serde::de::DeserializeOwned,
{
    match cast::<R>(req) {
        Ok((id, params)) => {
            let mut server = server.lock().expect("server mutex poisoned");
            Some(R::handle(&mut server, connection, id, params))
        }
        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        Err(ExtractError::MethodMismatch(_)) => None,
    }
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn take_cancelled_request(
    cancelled_requests: &Arc<Mutex<HashSet<RequestId>>>,
    id: &RequestId,
) -> bool {
    let mut cancelled = cancelled_requests
        .lock()
        .expect("cancelled_requests mutex poisoned");
    cancelled.remove(id)
}

fn remove_active_request(active_requests: &Arc<Mutex<HashSet<RequestId>>>, id: &RequestId) -> bool {
    let mut active = active_requests
        .lock()
        .expect("active_requests mutex poisoned");
    active.remove(id)
}

fn to_request_id(id: NumberOrString) -> Option<RequestId> {
    match id {
        NumberOrString::Number(number) => Some(RequestId::from(number)),
        NumberOrString::String(text) => Some(RequestId::from(text)),
    }
}

fn send_request_cancelled(connection: &Connection, id: RequestId) -> Result<(), Box<dyn Error>> {
    let response = lsp_server::Response::new_err(id, -32800, "Request cancelled".to_string());
    connection.sender.send(Message::Response(response))?;
    Ok(())
}
