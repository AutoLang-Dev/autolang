mod server;

#[cfg(test)]
mod tests;

use crate::server::{REPARSE_TRACE_NOTIFICATION, ReparseTrace, SYNTAX_TREE_REQUEST, Server};
use locale::tr;
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response, ResponseError};
use lsp_types::{
  DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  DidSaveTextDocumentParams, DocumentSymbolParams, DocumentSymbolResponse, InitializeParams,
  SemanticTokensDeltaParams, SemanticTokensParams,
  notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    Notification as _, PublishDiagnostics,
  },
  request::{
    DocumentSymbolRequest, Request as _, SemanticTokensFullDeltaRequest, SemanticTokensFullRequest,
  },
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
    SemanticTokensFullDeltaRequest::METHOD => {
      let (id, params) =
        req.extract::<SemanticTokensDeltaParams>(SemanticTokensFullDeltaRequest::METHOD)?;
      let tokens =
        server.semantic_tokens_delta(&params.text_document.uri, &params.previous_result_id);
      let result = to_value(tokens)?;
      let resp = Response::new_ok(id, result);
      conn.sender.send(Message::Response(resp))?;
    }
    DocumentSymbolRequest::METHOD => {
      let (id, params) = req.extract::<DocumentSymbolParams>(DocumentSymbolRequest::METHOD)?;
      let symbols = server
        .document_symbols(&params.text_document.uri)
        .map(DocumentSymbolResponse::Nested);
      let result = to_value(symbols)?;
      let resp = Response::new_ok(id, result);
      conn.sender.send(Message::Response(resp))?;
    }
    SYNTAX_TREE_REQUEST => {
      let Some(uri) = crate::server::syntax_tree_uri(req.params) else {
        conn.sender.send(Message::Response(Response {
          id: req.id,
          result: None,
          error: Some(ResponseError {
            code: ErrorCode::InvalidParams as i32,
            message: tr().lsp_document_not_found(),
            data: None,
          }),
        }))?;
        return Ok(());
      };

      let Some(tree) = server.syntax_tree(&uri) else {
        conn.sender.send(Message::Response(Response {
          id: req.id,
          result: None,
          error: Some(ResponseError {
            code: ErrorCode::InvalidParams as i32,
            message: tr().lsp_document_not_found(),
            data: None,
          }),
        }))?;
        return Ok(());
      };

      conn.sender.send(Message::Response(Response::new_ok(
        req.id,
        crate::server::syntax_tree_result(tree),
      )))?;
    }

    _ => conn.sender.send(Message::Response(Response {
      id: req.id,
      result: None,
      error: Some(ResponseError {
        code: ErrorCode::MethodNotFound as i32,
        message: tr().lsp_method_not_found(),
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
      publish_diagnostics(server, conn, &uri)?;
    }
    DidChangeTextDocument::METHOD => {
      let DidChangeTextDocumentParams {
        text_document,
        content_changes,
      } = from_value(not.params)?;
      let uri = text_document.uri;
      for change in content_changes {
        let trace = if let Some(range) = change.range {
          server.change_document_range(&uri, range, change.text)
        } else {
          server.change_document_full(&uri, change.text)
        };
        if let Some(trace) = trace {
          publish_reparse_trace(conn, trace)?;
        }
      }
      publish_diagnostics(server, conn, &uri)?;
    }
    DidSaveTextDocument::METHOD => {
      let DidSaveTextDocumentParams {
        text_document,
        text,
      } = from_value(not.params)?;
      let uri = text_document.uri;
      if let Some(trace) = server.save_document(&uri, text) {
        publish_reparse_trace(conn, trace)?;
      }
      publish_diagnostics(server, conn, &uri)?;
    }
    DidCloseTextDocument::METHOD => {
      let DidCloseTextDocumentParams { text_document } = from_value(not.params)?;
      let uri = text_document.uri;
      server.close_document(&uri);
      let params = Server::empty_diagnostics(uri);
      conn.sender.send(Message::Notification(Notification::new(
        PublishDiagnostics::METHOD.to_string(),
        params,
      )))?;
    }
    _ => (),
  }

  Ok(())
}

fn publish_reparse_trace(conn: &Connection, trace: ReparseTrace) -> anyhow::Result<()> {
  conn.sender.send(Message::Notification(Notification::new(
    REPARSE_TRACE_NOTIFICATION.to_string(),
    trace.to_json(),
  )))?;
  Ok(())
}

fn publish_diagnostics(
  server: &Server,
  conn: &Connection,
  uri: &lsp_types::Uri,
) -> anyhow::Result<()> {
  let params = server.diagnostics(uri);
  conn.sender.send(Message::Notification(Notification::new(
    PublishDiagnostics::METHOD.to_string(),
    params,
  )))?;
  Ok(())
}
