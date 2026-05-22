mod server;

use crate::server::Server;
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response, ResponseError};
use lsp_types::{
  DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  InitializeParams, SemanticTokensParams,
  notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
  },
  request::{Request as _, SemanticTokensFullRequest},
};
use serde_json::{from_value, to_value};

pub fn run() -> anyhow::Result<()> {
  let (conn, io_threads) = Connection::stdio();

  let capabilities = to_value(Server::capabilities())?;
  let init_params = conn.initialize(capabilities)?;
  let init_params: InitializeParams = from_value(init_params)?;

  let mut server = Server::new(init_params);

  for msg in &conn.receiver {
    match msg {
      Message::Request(req) => {
        if conn.handle_shutdown(&req)? {
          break;
        }
        handle_request(&mut server, &conn, req)?;
      }
      Message::Notification(not) => handle_notification(&mut server, &conn, not)?,
      _ => (),
    }
  }

  io_threads.join()?;
  Ok(())
}

pub fn handle_request(server: &mut Server, conn: &Connection, req: Request) -> anyhow::Result<()> {
  match req.method.as_str() {
    SemanticTokensFullRequest::METHOD => {
      let (id, params) = req.extract::<SemanticTokensParams>(SemanticTokensFullRequest::METHOD)?;
      let tokens = server.semantic_tokens(&params.text_document.uri);
      let result = to_value(tokens)?;
      let resp = Response::new_ok(id, result);
      conn.sender.send(Message::Response(resp))?;
    }

    _ => conn.sender.send(Message::Response(Response {
      id: req.id,
      result: None,
      error: Some(ResponseError {
        code: ErrorCode::MethodNotFound as i32,
        message: "method not found".to_string(),
        data: None,
      }),
    }))?,
  }

  Ok(())
}

pub fn handle_notification(
  server: &mut Server,
  conn: &Connection,
  not: Notification,
) -> anyhow::Result<()> {
  match not.method.as_str() {
    DidOpenTextDocument::METHOD => {
      let DidOpenTextDocumentParams { text_document } = from_value(not.params)?;
      let uri = text_document.uri;
      server.update_document(uri.clone(), text_document.text);
      server.publish_diagnostic(&uri, conn)?;
    }
    DidChangeTextDocument::METHOD => {
      let DidChangeTextDocumentParams {
        text_document,
        content_changes,
      } = from_value(not.params)?;
      if let Some(change) = content_changes.into_iter().last() {
        let uri = text_document.uri;
        server.update_document(uri.clone(), change.text);
        server.publish_diagnostic(&uri, conn)?;
      }
    }
    DidCloseTextDocument::METHOD => {
      let DidCloseTextDocumentParams { text_document } = from_value(not.params)?;
      server.close_document(&text_document.uri);
    }
    _ => (),
  }

  Ok(())
}
