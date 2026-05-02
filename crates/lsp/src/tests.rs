mod diagnostics;
mod document;
mod document_symbols;
mod reparse_trace;
mod semantic_tokens;
mod syntax_tree;

use crate::server::{SYNTAX_TREE_REQUEST, Server};
use lsp_server::{Request, RequestId};
use lsp_types::request::{DocumentSymbolRequest, Request as _};
use lsp_types::{InitializeParams, Uri};
use std::sync::Mutex;

static LOCALE_LOCK: Mutex<()> = Mutex::new(());

fn with_locale<T>(locale: &str, f: impl FnOnce() -> T) -> T {
  let _guard = LOCALE_LOCK.lock().unwrap_or_else(|err| err.into_inner());
  locale::set_tr(locale.to_string());
  let result = f();
  locale::set_tr("en-US".to_string());
  result
}

fn uri() -> Uri {
  "file:///test.auto".parse().unwrap()
}

fn server_with(text: &str) -> (Server, Uri) {
  let uri = uri();
  let mut server = Server::new(InitializeParams::default());
  server.update_document(uri.clone(), text.to_string());
  (server, uri)
}

fn syntax_tree_request(uri: &Uri) -> Request {
  Request::new(
    RequestId::from(1),
    SYNTAX_TREE_REQUEST.to_string(),
    serde_json::json!({ "textDocument": { "uri": uri } }),
  )
}

fn document_symbol_request(uri: &Uri) -> Request {
  Request::new(
    RequestId::from(1),
    DocumentSymbolRequest::METHOD.to_string(),
    serde_json::json!({ "textDocument": { "uri": uri } }),
  )
}

fn assert_full_trace(value: serde_json::Value) {
  assert_eq!(value["strategy"], "full");
  assert!(value.get("uri").is_some());
  assert!(value.get("editRange").is_none());
  assert!(value.get("dirtyRange").is_none());
  assert!(value.get("reason").is_none());
  assert_eq!(value.as_object().unwrap().len(), 2);
}
