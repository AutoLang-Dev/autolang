mod server;

use crate::server::{
  ReparseTrace, ReparseTraceParams, Server, SyntaxTreeParams, SyntaxTreeRequest,
};
use line_index::TextRange;
use locale::tr;
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response, ResponseError};
use lsp_types::{
  DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  DocumentSymbolParams, DocumentSymbolResponse, InitializeParams, SemanticTokensParams,
  notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
  },
  request::{DocumentSymbolRequest, Request as _, SemanticTokensFullRequest},
};
use serde_json::{from_value, to_value};
use syntax::Indel;

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

    SyntaxTreeRequest::METHOD => {
      let (id, params) = req.extract::<SyntaxTreeParams>(SyntaxTreeRequest::METHOD)?;
      let Some(tree) = server.syntax_tree(&params.text_document.uri) else {
        conn.sender.send(Message::Response(Response {
          id,
          result: None,
          error: Some(ResponseError {
            code: ErrorCode::InvalidParams as i32,
            message: tr().lsp_document_not_found(),
            data: None,
          }),
        }))?;
        return Ok(());
      };
      let result = to_value(tree)?;
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
      server.update_document(&uri, text_document.text);
      server.publish_diagnostic(&uri, conn)?;
    }
    DidChangeTextDocument::METHOD => {
      let DidChangeTextDocumentParams {
        text_document,
        content_changes,
      } = from_value(not.params)?;
      let uri = text_document.uri;
      let doc = server.get_document_mut(&uri).unwrap();
      let mut ranges = Vec::new();

      if content_changes.len() == 1 && content_changes[0].range.is_none() {
        doc.set_text(&content_changes[0].text);
      } else {
        for change in content_changes {
          let delete = doc.lsp_range_to_span(change.range.unwrap());
          let insert = change.text;
          let indel = Indel { delete, insert };

          if let Some(trace) = doc.apply_change(&indel) {
            assert!(trace.contains_range(indel.delete));

            let trace = TextRange::at(
              trace.start(),
              trace.len() - indel.delete.len() + indel.insert_len(),
            );
            let range = doc.span_to_lsp_range(trace);
            ranges.push(range);
          }
        }
      }

      server.publish_diagnostic(&uri, conn)?;

      conn.sender.send(Message::Notification(Notification::new(
        ReparseTrace::METHOD.to_string(),
        ReparseTraceParams::new(uri, ranges),
      )))?;
    }
    DidCloseTextDocument::METHOD => {
      let DidCloseTextDocumentParams { text_document } = from_value(not.params)?;
      server.close_document(&text_document.uri);
    }
    _ => (),
  }

  Ok(())
}
