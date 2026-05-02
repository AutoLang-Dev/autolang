use super::*;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
  InitializeParams, Position, Range,
  notification::{DidChangeTextDocument, Notification as _, PublishDiagnostics},
};

#[test]
fn range_change_returns_token_reparse_trace() {
  let (mut server, uri) = server_with("foo: mod;");

  let trace = server
    .change_document_range(
      &uri,
      Range::new(Position::new(0, 1), Position::new(0, 2)),
      "a".to_string(),
    )
    .unwrap();
  let value = trace.to_json();

  assert_eq!(value["strategy"], "token");
  assert_eq!(value["dirtyRange"]["old"]["start"]["line"], 0);
  assert_eq!(value["dirtyRange"]["old"]["start"]["character"], 0);
  assert_eq!(value["dirtyRange"]["old"]["end"]["line"], 0);
  assert_eq!(value["dirtyRange"]["old"]["end"]["character"], 3);
  assert_eq!(value["dirtyRange"]["new"]["start"]["character"], 0);
  assert_eq!(value["dirtyRange"]["new"]["end"]["character"], 3);
  assert!(value.get("oldKind").is_none());
  assert!(value.get("newKind").is_none());
  assert!(value.get("reparser").is_none());
  assert!(value.get("oldDirtyRange").is_none());
  assert!(value.get("newDirtyRange").is_none());
}

#[test]
fn range_change_returns_noop_reparse_trace() {
  let (mut server, uri) = server_with("foo: mod;");

  let trace = server
    .change_document_range(
      &uri,
      Range::new(Position::new(0, 0), Position::new(0, 3)),
      "foo".to_string(),
    )
    .unwrap();
  let value = trace.to_json();

  assert_eq!(value["strategy"], "noop");
  assert!(value.get("oldKind").is_none());
  assert!(value.get("newKind").is_none());
  assert!(value.get("dirtyRange").is_none());
  assert!(value.get("oldDirtyRange").is_none());
  assert!(value.get("newDirtyRange").is_none());
}

#[test]
fn range_change_returns_module_inner_reparse_trace_for_item_kind_mutation() {
  let (mut server, uri) = server_with("x: Int; y: Int;");

  let trace = server
    .change_document_range(
      &uri,
      Range::new(Position::new(0, 3), Position::new(0, 7)),
      "fn();".to_string(),
    )
    .unwrap();
  let value = trace.to_json();

  assert_eq!(value["strategy"], "node");
  assert_eq!(value["dirtyRange"]["old"]["start"]["line"], 0);
  assert_eq!(value["dirtyRange"]["new"]["start"]["line"], 0);
  assert_eq!(value["reparser"], "ModuleInner");
  assert_eq!(value["oldKind"], "ModuleInner");
  assert_eq!(value["newKind"], "ModuleInner");
  assert!(value.get("oldDirtyRange").is_none());
  assert!(value.get("newDirtyRange").is_none());
}

#[test]
fn range_change_returns_full_trace_for_unbalanced_delimiters() {
  let (mut server, uri) = server_with("x: = { a; };");

  let trace = server
    .change_document_range(
      &uri,
      Range::new(Position::new(0, 7), Position::new(0, 7)),
      "{".to_string(),
    )
    .unwrap();
  let value = trace.to_json();

  assert_full_trace(value);
}

#[test]
fn range_change_returns_full_trace_for_reparse_miss() {
  let (mut server, uri) = server_with("#!/usr/bin/env autolang\nfoo: mod;");

  let trace = server
    .change_document_range(
      &uri,
      Range::new(Position::new(0, 0), Position::new(0, 2)),
      "##".to_string(),
    )
    .unwrap();
  let value = trace.to_json();

  assert_full_trace(value);
}

#[test]
fn range_change_returns_utf16_reparse_trace_ranges() {
  let (mut server, uri) = server_with("x: = \"好a\";");

  let trace = server
    .change_document_range(
      &uri,
      Range::new(Position::new(0, 7), Position::new(0, 8)),
      "b".to_string(),
    )
    .unwrap();
  let value = trace.to_json();

  assert_eq!(value["strategy"], "token");
  assert_eq!(
    value["editRange"]["start"],
    serde_json::json!({ "line": 0, "character": 7 })
  );
  assert_eq!(
    value["editRange"]["end"],
    serde_json::json!({ "line": 0, "character": 8 })
  );
  assert_eq!(value["dirtyRange"]["old"]["start"]["character"], 5);
  assert_eq!(value["dirtyRange"]["old"]["end"]["character"], 9);
  assert_eq!(value["dirtyRange"]["new"]["start"]["character"], 5);
  assert_eq!(value["dirtyRange"]["new"]["end"]["character"], 9);
}

#[test]
fn did_change_sends_trace_for_each_content_change() {
  let uri = uri();
  let mut server = Server::new(InitializeParams::default());
  server.update_document(uri.clone(), "foo: mod;\nbar: mod;".to_string());
  let (server_conn, client_conn) = Connection::memory();

  crate::handle_notification(
    &mut server,
    &server_conn,
    Notification::new(
      DidChangeTextDocument::METHOD.to_string(),
      serde_json::json!({
        "textDocument": { "uri": uri, "version": 1 },
        "contentChanges": [
          {
            "range": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 0, "character": 3 }
            },
            "text": "baz"
          },
          {
            "range": {
              "start": { "line": 1, "character": 0 },
              "end": { "line": 1, "character": 3 }
            },
            "text": "qux"
          }
        ]
      }),
    ),
  )
  .unwrap();

  let mut traces = 0;
  let mut diagnostics = 0;
  while let Ok(message) = client_conn.receiver.try_recv() {
    if let Message::Notification(notification) = message {
      if notification.method == crate::server::REPARSE_TRACE_NOTIFICATION {
        traces += 1;
      }
      if notification.method == PublishDiagnostics::METHOD {
        diagnostics += 1;
      }
    }
  }

  assert_eq!(traces, 2);
  assert_eq!(diagnostics, 1);
}
