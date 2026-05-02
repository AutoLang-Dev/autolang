use super::*;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
  InitializeParams, Position, Range, SaveOptions, TextDocumentSyncCapability, TextDocumentSyncKind,
  TextDocumentSyncSaveOptions,
  notification::{DidSaveTextDocument, Notification as _, PublishDiagnostics},
};

#[test]
fn capabilities_request_incremental_sync_with_save_text() {
  let capabilities = Server::capabilities();
  let Some(TextDocumentSyncCapability::Options(options)) = capabilities.text_document_sync else {
    panic!("expected text document sync options");
  };

  assert_eq!(options.open_close, Some(true));
  assert_eq!(options.change, Some(TextDocumentSyncKind::INCREMENTAL));
  assert_eq!(
    options.save,
    Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
      include_text: Some(true),
    })),
  );
}

#[test]
fn incremental_change_updates_document_and_diagnostics() {
  let (mut server, uri) = server_with("foo: mod;");

  server.change_document_range(
    &uri,
    Range::new(Position::new(0, 0), Position::new(0, 3)),
    "bar".to_string(),
  );

  let document = server.document(&uri).unwrap();
  assert_eq!(document.text, "bar: mod;");
  assert!(server.diagnostics(&uri).diagnostics.is_empty());
}

#[test]
fn thin_arrow_repair_clears_diagnostics() {
  let (mut server, uri) = server_with("outer: fn = {\n  inner: fn -> () = {};\n};");

  server.change_document_range(
    &uri,
    Range::new(Position::new(1, 12), Position::new(1, 13)),
    "".to_string(),
  );
  assert!(!server.diagnostics(&uri).diagnostics.is_empty());

  server.change_document_range(
    &uri,
    Range::new(Position::new(1, 12), Position::new(1, 12)),
    "-".to_string(),
  );
  assert!(server.diagnostics(&uri).diagnostics.is_empty());
}

#[test]
fn eof_semicolon_repair_clears_diagnostics() {
  let (mut server, uri) = server_with("root: mod\n");
  assert!(!server.diagnostics(&uri).diagnostics.is_empty());

  server.change_document_range(
    &uri,
    Range::new(Position::new(1, 0), Position::new(1, 0)),
    ";".to_string(),
  );

  assert!(server.diagnostics(&uri).diagnostics.is_empty());
}

#[test]
fn save_document_with_text_replaces_text_and_clears_diagnostics() {
  let (mut server, uri) = server_with("root: mod\n");
  assert!(!server.diagnostics(&uri).diagnostics.is_empty());

  let trace = server
    .save_document(&uri, Some("root: mod;\n".to_string()))
    .unwrap();
  let value = trace.to_json();

  assert_eq!(server.document(&uri).unwrap().text, "root: mod;\n");
  assert!(server.diagnostics(&uri).diagnostics.is_empty());
  assert_full_trace(value);
}

#[test]
fn full_document_change_returns_full_trace_without_edit_range() {
  let (mut server, uri) = server_with("root: mod\n");

  let trace = server
    .change_document_full(&uri, "root: mod;\n".to_string())
    .unwrap();
  let value = trace.to_json();

  assert_eq!(server.document(&uri).unwrap().text, "root: mod;\n");
  assert!(server.diagnostics(&uri).diagnostics.is_empty());
  assert_full_trace(value);
}

#[test]
fn save_document_with_text_updates_diagnostics() {
  let (mut server, uri) = server_with("root: mod;\n");
  assert!(server.diagnostics(&uri).diagnostics.is_empty());

  server.save_document(&uri, Some("root: mod\n".to_string()));

  assert!(!server.diagnostics(&uri).diagnostics.is_empty());
}

#[test]
fn save_document_without_text_reparses_current_text() {
  let (mut server, uri) = server_with("root: mod;\n");

  let trace = server.save_document(&uri, None).unwrap();
  let value = trace.to_json();

  assert_eq!(server.document(&uri).unwrap().text, "root: mod;\n");
  assert!(server.diagnostics(&uri).diagnostics.is_empty());
  assert_full_trace(value);
}

#[test]
fn did_save_sends_full_trace_and_diagnostics() {
  let uri = uri();
  let mut server = Server::new(InitializeParams::default());
  server.update_document(uri.clone(), "root: mod\n".to_string());
  let (server_conn, client_conn) = Connection::memory();

  crate::handle_notification(
    &mut server,
    &server_conn,
    Notification::new(
      DidSaveTextDocument::METHOD.to_string(),
      serde_json::json!({
        "textDocument": { "uri": uri },
        "text": "root: mod;\n"
      }),
    ),
  )
  .unwrap();

  let mut saw_full_trace = false;
  let mut saw_diagnostics = false;
  while let Ok(message) = client_conn.receiver.try_recv() {
    if let Message::Notification(notification) = message {
      if notification.method == crate::server::REPARSE_TRACE_NOTIFICATION {
        saw_full_trace = notification.params["strategy"] == "full"
          && notification.params.get("editRange").is_none()
          && notification.params.get("dirtyRange").is_none();
      }
      if notification.method == PublishDiagnostics::METHOD {
        saw_diagnostics = true;
      }
    }
  }

  assert!(saw_full_trace);
  assert!(saw_diagnostics);
  assert!(server.diagnostics(&uri).diagnostics.is_empty());
}
