use lsp_server::{RequestId, Response};
use lsp_types::request::{DocumentSymbolRequest, GotoDefinition, HoverRequest, Request};

use crate::server::Server;

pub(crate) trait RequestHandler: Request {
    fn handle(server: &mut Server, id: RequestId, params: Self::Params) -> Response;
}

impl RequestHandler for GotoDefinition {
    fn handle(server: &mut Server, id: RequestId, params: Self::Params) -> Response {
        eprintln!("got gotoDefinition request #{}: {:?}", id, params);
        let result = server.handle_goto_definition(params);
        let result = serde_json::to_value(&result).unwrap();
        let resp = Response {
            id,
            result: Some(result),
            error: None,
        };
        eprintln!("got gotoDefinition response {:?}", resp);
        resp
    }
}

impl RequestHandler for HoverRequest {
    fn handle(server: &mut Server, id: RequestId, params: Self::Params) -> Response {
        eprintln!("got hover request #{}: {:?}", id, params);
        let result = server.handle_hover(params);
        let result = serde_json::to_value(&result).unwrap();
        let resp = Response {
            id,
            result: Some(result),
            error: None,
        };
        eprintln!("got hover response {:?}", resp);
        resp
    }
}

impl RequestHandler for DocumentSymbolRequest {
    fn handle(server: &mut Server, id: RequestId, params: Self::Params) -> Response {
        eprintln!("got document symbols request #{}: {:?}", id, params);
        let result = server.handle_document_symbols(params);
        let result = serde_json::to_value(&result).unwrap();
        let resp = Response {
            id,
            result: Some(result),
            error: None,
        };
        eprintln!("got document symbols response {:?}", resp);
        resp
    }
}
