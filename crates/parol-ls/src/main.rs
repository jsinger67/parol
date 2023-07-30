use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::{cell::RefCell, error::Error, result::Result};

use crate::handler::RequestHandler;
use crate::{arguments::Arguments, config::Config};

pub mod arguments;
mod config;
mod convert_to_rng;
pub mod diagnostics;
pub mod document_state;
pub mod errors;
mod formatting;
mod handler;
pub mod parol_ls_grammar;
mod parol_ls_grammar_trait;
mod parol_ls_parser;
mod rng;
mod server;
mod utils;

extern crate clap;
extern crate parol_runtime;

use clap::Parser;

use errors::ServerError;
use lsp_server::{Connection, ExtractError, Message, Request, RequestId};
use lsp_types::notification::DidChangeConfiguration;
use lsp_types::request::RegisterCapability;
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification,
    },
    request::{
        DocumentSymbolRequest, Formatting, GotoDefinition, HoverRequest, PrepareRenameRequest,
        Rename,
    },
    HoverProviderCapability, InitializeParams, OneOf, RenameOptions, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};
use lsp_types::{Registration, RegistrationParams};
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

macro_rules! request_match {
    ($req_type:ty, $server:expr, $connection:expr, $req:expr) => {
        match cast::<$req_type>($req) {
            Ok((id, params)) => {
                let resp = <$req_type>::handle(&mut $server.borrow_mut(), id, params);
                // eprintln!("send response: {:?}", resp);
                $connection.sender.send(Message::Response(resp))?;
                continue;
            }
            Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
            Err(ExtractError::MethodMismatch(req)) => req,
        };
    };
}

// Note that this function only sends the request. The response handling is done in the main loop!
pub(crate) fn send_request<R>(
    conn: Arc<Connection>,
    params: R::Params,
    _server: &RefCell<Server>,
) -> Result<(), ServerError>
where
    R: lsp_types::request::Request,
    R::Params: Serialize,
{
    let r = Request::new(RequestCounter::next(), R::METHOD.to_string(), params);
    conn.sender
        .send(r.into())
        .map_err(|err| ServerError::ProtocolError { err: Box::new(err) })
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
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();

    let initialization_params: InitializeParams =
        serde_json::from_value(connection.initialize(server_capabilities)?).unwrap();
    let config = Config::new(initialization_params, args);

    main_loop(connection, config)?;
    io_threads.join()?;

    eprintln!("shutting down parol language server");
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
    let server = RefCell::new(server::Server::new(config.lookahead()));
    // Then update properties from client configuration (i.e. settings).
    server
        .borrow_mut()
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
            &server,
        )?;
    }

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                eprintln!("got request: {:?}", req);
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                match req.method.as_str() {
                    <GotoDefinition as lsp_types::request::Request>::METHOD => {
                        request_match!(GotoDefinition, server, connection, req);
                    }
                    <HoverRequest as lsp_types::request::Request>::METHOD => {
                        request_match!(HoverRequest, server, connection, req);
                    }
                    <DocumentSymbolRequest as lsp_types::request::Request>::METHOD => {
                        request_match!(DocumentSymbolRequest, server, connection, req);
                    }
                    <PrepareRenameRequest as lsp_types::request::Request>::METHOD => {
                        request_match!(PrepareRenameRequest, server, connection, req);
                    }
                    <Rename as lsp_types::request::Request>::METHOD => {
                        request_match!(Rename, server, connection, req);
                    }
                    <Formatting as lsp_types::request::Request>::METHOD => {
                        request_match!(Formatting, server, connection, req);
                    }
                    _ => {
                        eprintln!("Unhandled request {}", req.method);
                    }
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                process_notification(not, connection.clone(), &server)?;
            }
        }
    }
    Ok(())
}

fn process_notification(
    not: lsp_server::Notification,
    connection: Arc<Connection>,
    server: &RefCell<Server>,
) -> Result<(), Box<dyn Error>> {
    eprintln!("got notification: {:?}", not);
    match not.method.as_str() {
        DidOpenTextDocument::METHOD => server.borrow_mut().handle_open_document(connection, not)?,
        DidChangeTextDocument::METHOD => server
            .borrow_mut()
            .handle_change_document(connection, not)?,
        DidCloseTextDocument::METHOD => server.borrow_mut().handle_close_document(not)?,
        DidChangeConfiguration::METHOD => server.borrow_mut().handle_changed_configuration(not)?,
        _ => {}
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
