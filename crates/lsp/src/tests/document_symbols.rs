use super::*;
use lsp_server::{Connection, Message};
use lsp_types::{DocumentSymbolResponse, OneOf, Position, Range, SymbolKind};

#[test]
fn capabilities_enable_document_symbols() {
  let capabilities = Server::capabilities();

  assert_eq!(
    capabilities.document_symbol_provider,
    Some(OneOf::Left(true))
  );
}

#[test]
fn document_symbols_return_nested_items_and_fields() {
  let (server, uri) = server_with(
    "root: mod = { x: Int = 1; add: fn(a: Int) -> Int = a; Point: type = { x: Int, 0: Int }; child: mod; };",
  );

  let symbols = server.document_symbols(&uri).unwrap();

  assert_eq!(symbols.len(), 1);
  assert_eq!(symbols[0].name, "root");
  assert_eq!(symbols[0].kind, SymbolKind::MODULE);

  let children = symbols[0].children.as_ref().unwrap();
  assert_eq!(children.len(), 4);
  assert_eq!(children[0].name, "x");
  assert_eq!(children[0].kind, SymbolKind::VARIABLE);
  assert_eq!(children[1].name, "add");
  assert_eq!(children[1].kind, SymbolKind::FUNCTION);
  assert!(children[1].children.is_none());
  assert_eq!(children[2].name, "Point");
  assert_eq!(children[2].kind, SymbolKind::STRUCT);
  assert_eq!(children[3].name, "child");
  assert_eq!(children[3].kind, SymbolKind::MODULE);

  let fields = children[2].children.as_ref().unwrap();
  assert_eq!(fields.len(), 2);
  assert_eq!(fields[0].name, "x");
  assert_eq!(fields[0].kind, SymbolKind::FIELD);
  assert_eq!(fields[1].name, "0");
  assert_eq!(fields[1].kind, SymbolKind::FIELD);
}

#[test]
fn document_symbols_skip_using_empty_and_parameters() {
  let (server, uri) = server_with("using foo; ; add: fn(a: Int, b: Int) -> Int = a;");

  let symbols = server.document_symbols(&uri).unwrap();

  assert_eq!(symbols.len(), 1);
  assert_eq!(symbols[0].name, "add");
  assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
  assert!(symbols[0].children.is_none());
}

#[test]
fn document_symbols_track_change_and_save() {
  let (mut server, uri) = server_with("foo: mod;");
  assert_eq!(server.document_symbols(&uri).unwrap()[0].name, "foo");

  server.change_document_range(
    &uri,
    Range::new(Position::new(0, 0), Position::new(0, 3)),
    "bar".to_string(),
  );
  assert_eq!(server.document_symbols(&uri).unwrap()[0].name, "bar");

  server.save_document(&uri, Some("baz: mod;".to_string()));
  assert_eq!(server.document_symbols(&uri).unwrap()[0].name, "baz");
}

#[test]
fn document_symbols_use_utf16_ranges() {
  let (server, uri) = server_with("pre: mod; 好: mod;");

  let symbols = server.document_symbols(&uri).unwrap();

  assert_eq!(symbols[1].name, "好");
  assert_eq!(symbols[1].selection_range.start, Position::new(0, 10));
  assert_eq!(symbols[1].selection_range.end, Position::new(0, 11));
}

#[test]
fn document_symbol_request_returns_nested_response() {
  let uri = uri();
  let mut server = Server::new(InitializeParams::default());
  server.update_document(uri.clone(), "foo: mod;".to_string());
  let (server_conn, client_conn) = Connection::memory();

  crate::handle_request(&mut server, &server_conn, document_symbol_request(&uri)).unwrap();

  let Message::Response(response) = client_conn.receiver.recv().unwrap() else {
    panic!("expected response");
  };
  let result = response.result.unwrap();
  let response: DocumentSymbolResponse = serde_json::from_value(result).unwrap();
  let DocumentSymbolResponse::Nested(symbols) = response else {
    panic!("expected nested document symbols");
  };

  assert_eq!(symbols.len(), 1);
  assert_eq!(symbols[0].name, "foo");
}

#[test]
fn document_symbol_request_returns_null_for_missing_document() {
  let uri = uri();
  let mut server = Server::new(InitializeParams::default());
  let (server_conn, client_conn) = Connection::memory();

  crate::handle_request(&mut server, &server_conn, document_symbol_request(&uri)).unwrap();

  let Message::Response(response) = client_conn.receiver.recv().unwrap() else {
    panic!("expected response");
  };

  assert_eq!(response.result, Some(serde_json::Value::Null));
  assert!(response.error.is_none());
}
