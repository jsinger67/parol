use std::{cell::RefCell, error::Error, result::Result};

use crate::arguments::Config;
use crate::handler::RequestHandler;

pub mod arguments;
mod ast;
pub mod diagnostics;
pub mod document_state;
pub mod errors;
mod handler;
pub mod parol_ls_grammar;
mod parol_ls_grammar_trait;
mod parol_ls_parser;
mod rng;
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
use lsp_server::{Connection, ExtractError, Message, Request, RequestId};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification,
    },
    request::{DocumentSymbolRequest, GotoDefinition, HoverRequest, PrepareRenameRequest, Rename},
    HoverProviderCapability, InitializeParams, OneOf, RenameOptions, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};

macro_rules! request_match {
    ($req_type:ty, $server:expr, $connection:expr, $req:expr) => {
        match cast::<$req_type>($req) {
            Ok((id, params)) => {
                let resp = <$req_type>::handle(&mut $server.borrow_mut(), id, params);
                $connection.sender.send(Message::Response(resp))?;
                continue;
            }
            Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
            Err(ExtractError::MethodMismatch(req)) => req,
        };
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    debug!("env logger started");

    let config = Config::parse();
    eprintln!("Starting parol language server");
    let (connection, io_threads) = Connection::connect((config.ip_address, config.port_number))?;

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(&connection, initialization_params, config)?;
    io_threads.join()?;

    eprintln!("shutting down parol language server");
    Ok(())
}

fn main_loop(
    connection: &Connection,
    params: serde_json::Value,
    config: Config,
) -> Result<(), Box<dyn Error>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let server = RefCell::new(server::Server::new(config.lookahead));
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", req);
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
                    _ => {}
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                eprintln!("got notification: {:?}", not);
                match not.method.as_str() {
                    DidOpenTextDocument::METHOD => {
                        server.borrow_mut().handle_open_document(connection, not)?
                    }
                    DidChangeTextDocument::METHOD => server
                        .borrow_mut()
                        .handle_change_document(connection, not)?,
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
