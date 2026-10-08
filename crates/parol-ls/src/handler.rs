use std::sync::Arc;

use lsp_server::{RequestId, Response};
use lsp_types::request::{
    CodeActionRequest, DocumentSymbolRequest, Formatting, GotoDefinition, HoverRequest,
    PrepareRenameRequest, Rename, Request,
};

use crate::server::Server;

pub(crate) trait RequestHandler: Request {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response;
}

impl RequestHandler for GotoDefinition {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response {
        eprintln!(
            "got gotoDefinition request #{id}: uri={:?}",
            params.text_document_position_params.text_document.uri
        );
        let result = server.handle_goto_definition(params, connection);
        let result = serde_json::to_value(result).unwrap();
        Response::new_ok(id, result)
    }
}

impl RequestHandler for HoverRequest {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response {
        eprintln!(
            "got hover request #{id}: uri={:?}",
            params.text_document_position_params.text_document.uri
        );
        let result = server.handle_hover(params, connection);
        let result = serde_json::to_value(result).unwrap();
        Response::new_ok(id, result)
    }
}

impl RequestHandler for DocumentSymbolRequest {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response {
        eprintln!(
            "got document symbols request #{id}: uri={:?}",
            params.text_document.uri
        );
        let result = server.handle_document_symbols(params, connection);
        let result = serde_json::to_value(result).unwrap();
        Response::new_ok(id, result)
    }
}

impl RequestHandler for PrepareRenameRequest {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response {
        eprintln!("got prepare rename request #{id}: {params:?}");
        let result = server.handle_prepare_rename(params, connection);
        let result = serde_json::to_value(result).unwrap();
        Response::new_ok(id, result)
    }
}

impl RequestHandler for Rename {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response {
        eprintln!("got rename request #{id}: {params:?}");
        let result = server.handle_rename(params, connection);
        let result = serde_json::to_value(result).unwrap();
        Response::new_ok(id, result)
    }
}

impl RequestHandler for Formatting {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response {
        eprintln!("got formatting request #{id}: {params:?}");
        let result = server.handle_formatting(params, connection);
        let result = serde_json::to_value(result).unwrap();
        Response::new_ok(id, result)
    }
}

impl RequestHandler for CodeActionRequest {
    fn handle(
        server: &mut Server,
        connection: Arc<lsp_server::Connection>,
        id: RequestId,
        params: Self::Params,
    ) -> Response {
        eprintln!("got codeAction request #{id}: {params:?}");
        let result = server.handle_code_action(params, connection);
        let result = serde_json::to_value(result).unwrap();
        Response::new_ok(id, result)
    }
}
