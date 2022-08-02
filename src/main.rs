use std::{cell::RefCell, env, error::Error, result::Result};

use crate::arguments::Config;

pub mod arguments;
mod errors;
pub mod parol_ls_grammar;
mod parol_ls_grammar_trait;
mod parol_ls_parser;
mod server;
mod utils;

#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate function_name;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate thiserror;

extern crate clap;
extern crate parol_runtime;

use clap::Parser;

use log::debug;
use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification,
    },
    request::GotoDefinition,
    InitializeParams, OneOf, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    debug!("env logger started");
    eprintln!("Args: {:?}", env::args());

    let config = Config::parse();
    eprintln!("Starting parol language server");
    let (connection, io_threads) = Connection::connect((config.ip_address, config.port_number))?;

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(&connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("shutting down parol language server");
    Ok(())
}

fn main_loop(connection: &Connection, params: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let server = RefCell::new(server::Server::new());
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", req);
                match cast::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        eprintln!("got gotoDefinition request #{}: {:?}", id, params);
                        let result = server.borrow_mut().handle_goto_definition(params);
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        eprintln!("got gotoDefinition response {:?}", resp);
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                eprintln!("got notification: {:?}", not);
                match not.method.as_str() {
                    DidOpenTextDocument::METHOD => {
                        server.borrow_mut().handle_open_document(&connection, not)?
                    }
                    DidChangeTextDocument::METHOD => server
                        .borrow_mut()
                        .handle_change_document(&connection, not)?,
                    DidCloseTextDocument::METHOD => {
                        server.borrow_mut().handle_close_document(not)?
                    }
                    _ => {}
                }
            }
        }
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
