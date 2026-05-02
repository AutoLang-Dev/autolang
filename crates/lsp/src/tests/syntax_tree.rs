use super::*;
use lsp_server::{Connection, Message};
use lsp_types::{InitializeParams, Position, Range};

#[test]
fn syntax_tree_request_returns_current_tree() {
  let uri = uri();
  let mut server = Server::new(InitializeParams::default());
  server.update_document(uri.clone(), "foo: mod;".to_string());
  let (server_conn, client_conn) = Connection::memory();

  crate::handle_request(&mut server, &server_conn, syntax_tree_request(&uri)).unwrap();

  let Message::Response(response) = client_conn.receiver.recv().unwrap() else {
    panic!("expected response");
  };
  let result = response.result.unwrap();
  let tree = result["tree"].as_str().unwrap();
  assert!(tree.contains("SourceFile"));
  assert!(tree.contains("ModuleItem"));
  assert!(tree.contains("Ident \"foo\""));
}

#[test]
fn syntax_tree_request_includes_errors() {
  let (server, uri) = server_with("foo fn;");
  let tree = server.syntax_tree(&uri).unwrap();

  assert!(tree.contains("Errors:"));
  assert!(tree.contains("Expected"));
}

#[test]
fn syntax_tree_request_reports_missing_document() {
  with_locale("zh-Hans", || {
    let uri = uri();
    let mut server = Server::new(InitializeParams::default());
    let (server_conn, client_conn) = Connection::memory();

    crate::handle_request(&mut server, &server_conn, syntax_tree_request(&uri)).unwrap();

    let Message::Response(response) = client_conn.receiver.recv().unwrap() else {
      panic!("expected response");
    };
    let error = response.error.unwrap();
    assert_eq!(error.code, lsp_server::ErrorCode::InvalidParams as i32);
    assert_eq!(error.message, "找不到文档");
  });
}

#[test]
fn syntax_tree_request_tracks_change_and_save() {
  let (mut server, uri) = server_with("foo: mod;");
  assert!(server.syntax_tree(&uri).unwrap().contains("Ident \"foo\""));

  server.change_document_range(
    &uri,
    Range::new(Position::new(0, 0), Position::new(0, 3)),
    "bar".to_string(),
  );
  assert!(server.syntax_tree(&uri).unwrap().contains("Ident \"bar\""));

  server.save_document(&uri, Some("baz: mod;".to_string()));
  assert!(server.syntax_tree(&uri).unwrap().contains("Ident \"baz\""));
}
