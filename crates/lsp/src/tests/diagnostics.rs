use super::*;
use lsp_types::Position;

#[test]
fn valid_module_has_no_diagnostics() {
  let (server, uri) = server_with("foo: mod;");
  assert!(server.diagnostics(&uri).diagnostics.is_empty());
}

#[test]
fn invalid_module_reports_token_range() {
  let (server, uri) = server_with("foo fn;");
  let diagnostics = server.diagnostics(&uri).diagnostics;

  assert!(diagnostics.iter().any(|diagnostic| {
    diagnostic.range.start == Position::new(0, 0) && diagnostic.range.end == Position::new(0, 3)
  }));
}

#[test]
fn diagnostics_are_localized() {
  with_locale("zh-Hans", || {
    let (server, uri) = server_with("root: mod\n");
    let diagnostics = server.diagnostics(&uri).diagnostics;

    assert!(
      diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "期望 ;，但遇到 文件结尾")
    );
  });
}

#[test]
fn close_document_clears_diagnostics() {
  let uri = uri();
  let params = Server::empty_diagnostics(uri.clone());

  assert_eq!(params.uri, uri);
  assert!(params.diagnostics.is_empty());
}
