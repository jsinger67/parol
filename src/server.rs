use std::{collections::HashMap, error::Error};

use lsp_server::Message;
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification,
        PublishDiagnostics,
    },
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    GotoDefinitionParams, GotoDefinitionResponse, Location, NumberOrString, Position,
    PublishDiagnosticsParams, TextDocumentContentChangeEvent, Url,
};
use miette::miette;

use crate::{parol_ls_grammar::ParolLsGrammar, parol_ls_parser::parse};

#[derive(Debug, Default)]
pub(crate) struct DocumentState {
    input: String,
    parsed_data: ParolLsGrammar,
}

impl DocumentState {
    fn ident_at_position(&self, position: Position) -> String {
        eprintln!("non_terminals: {:?}", self.parsed_data.non_terminals);
        if let Some((_, non_terminal)) = self
            .parsed_data
            .non_terminals
            .iter()
            .find(|(k, _)| k.start <= position && k.end > position)
        {
            non_terminal.clone()
        } else {
            String::default()
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Server {
    documents: HashMap<String, DocumentState>,
}

impl Server {
    pub(crate) fn try_parse(&self, uri: &Url) -> miette::Result<ParolLsGrammar> {
        let file_path = uri
            .to_file_path()
            .map_err(|_| miette!("Failed interpreting file path {}", uri.path()))?;
        let mut parol_ls_grammar = ParolLsGrammar::new();
        let document_state = self.documents.get(uri.path()).unwrap();
        eprintln!("try_parse");
        parse(&document_state.input, &file_path, &mut parol_ls_grammar)?;
        eprintln!("parol_ls_grammar: {:#?}", parol_ls_grammar);
        Ok(parol_ls_grammar)
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn handle_open_document(
        &mut self,
        connection: &&lsp_server::Connection,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidOpenTextDocumentParams = n.extract(DidOpenTextDocument::METHOD)?;
        self.documents.insert(
            params.text_document.uri.path().to_string(),
            DocumentState {
                input: params.text_document.text.clone(),
                ..Default::default()
            },
        );
        let parsed_data = self.try_parse(&params.text_document.uri);
        match parsed_data {
            Ok(parsed_data) => {
                self.documents
                    .get_mut(params.text_document.uri.path())
                    .unwrap()
                    .parsed_data = parsed_data;
                let result = PublishDiagnosticsParams::new(params.text_document.uri, vec![], None);
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
            Err(err) => {
                let result = PublishDiagnosticsParams::new(
                    params.text_document.uri,
                    Self::to_diagnostics(err),
                    None,
                );
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
        }
        Ok(())
    }

    pub(crate) fn handle_change_document(
        &mut self,
        connection: &lsp_server::Connection,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidChangeTextDocumentParams = n.extract(DidChangeTextDocument::METHOD)?;
        self.apply_changes(params.text_document.uri.path(), &params.content_changes);
        let parsed_data = self.try_parse(&params.text_document.uri);
        match parsed_data {
            Ok(parsed_data) => {
                self.documents
                    .get_mut(params.text_document.uri.path())
                    .unwrap()
                    .parsed_data = parsed_data;
                let result = PublishDiagnosticsParams::new(params.text_document.uri, vec![], None);
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
            Err(err) => {
                let result = PublishDiagnosticsParams::new(
                    params.text_document.uri,
                    Self::to_diagnostics(err),
                    None,
                );
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
        }
        Ok(())
    }

    pub(crate) fn handle_close_document(
        &mut self,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidCloseTextDocumentParams = n.extract(DidCloseTextDocument::METHOD)?;
        self.cleanup(params.text_document.uri.path());
        Ok(())
    }

    pub(crate) fn handle_goto_definition(
        &mut self,
        params: GotoDefinitionParams,
    ) -> GotoDefinitionResponse {
        let document_state = self
            .documents
            .get(
                params
                    .text_document_position_params
                    .text_document
                    .uri
                    .path(),
            )
            .unwrap();
        let text_at_position =
            document_state.ident_at_position(params.text_document_position_params.position);
        eprintln!("text_at_position: {}", text_at_position);
        let mut locations = Vec::new();
        if let Some(non_terminal_definitions) = document_state
            .parsed_data
            .non_terminal_definitions
            .get(&text_at_position)
        {
            for range in non_terminal_definitions {
                locations.push(Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: range.clone(),
                });
            }
        }
        GotoDefinitionResponse::Array(locations)
    }

    fn cleanup(&mut self, file_path: &str) {
        self.documents.remove(file_path);
    }

    fn apply_changes(
        &mut self,
        file_path: &str,
        content_changes: &[TextDocumentContentChangeEvent],
    ) {
        if let Some(change) = content_changes.last() {
            self.apply_change(file_path, change);
        }
    }

    fn apply_change(&mut self, file_path: &str, change: &TextDocumentContentChangeEvent) {
        self.documents.get_mut(file_path).unwrap().input = change.text.clone();
    }

    fn to_diagnostics(err: miette::ErrReport) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if let Some(parser_err) = err.downcast_ref::<parol_runtime::errors::ParserError>() {
            let parser_err_diag: Box<&dyn miette::Diagnostic> = Box::new(parser_err);
            let diagnostic = Diagnostic {
                source: Some(
                    parser_err
                        .source()
                        .map_or(String::default(), |e| e.to_string()),
                ),
                code: Some(NumberOrString::String(
                    parser_err_diag
                        .code()
                        .map_or("Unknown error code".to_string(), |d| d.to_string()),
                )),
                message: parser_err_diag.to_string(),
                ..Default::default()
            };
            diagnostics.push(diagnostic);
        }
        diagnostics
    }
}
