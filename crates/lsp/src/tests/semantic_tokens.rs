use super::*;
use crate::server::TokenType;
use lsp_types::{Position, Range, SemanticTokensFullDeltaResult};

#[test]
fn semantic_tokens_come_from_syntax_tree() {
  let (mut server, uri) = server_with("foo: mod; // x");
  let tokens = server.semantic_tokens(&uri).data;
  let token_types = tokens
    .iter()
    .map(|token| token.token_type)
    .collect::<Vec<_>>();

  assert_eq!(
    token_types,
    vec![
      TokenType::Module as u32,
      TokenType::Operator as u32,
      TokenType::Keyword as u32,
      TokenType::Operator as u32,
      TokenType::Comment as u32,
    ],
  );
}

#[test]
fn semantic_tokens_keep_shebang_as_comment() {
  let (mut server, uri) = server_with("#!/usr/bin/env autolang\nfoo: mod;");
  let tokens = server.semantic_tokens(&uri).data;
  let token_types = tokens
    .iter()
    .map(|token| token.token_type)
    .collect::<Vec<_>>();

  assert_eq!(
    token_types,
    vec![
      TokenType::Comment as u32,
      TokenType::Module as u32,
      TokenType::Operator as u32,
      TokenType::Keyword as u32,
      TokenType::Operator as u32,
    ],
  );
}

#[test]
fn semantic_tokens_classify_item_contexts() {
  let (mut server, uri) =
    server_with("x: Int = 42; add: fn(a: Int) -> Int = a; Point: type = { x: Int };");
  let tokens = server.semantic_tokens(&uri).data;
  let token_types = tokens
    .iter()
    .map(|token| token.token_type)
    .collect::<Vec<_>>();

  assert_eq!(
    token_types,
    vec![
      TokenType::Ident as u32,
      TokenType::Operator as u32,
      TokenType::Type as u32,
      TokenType::Operator as u32,
      TokenType::Number as u32,
      TokenType::Operator as u32,
      TokenType::Function as u32,
      TokenType::Operator as u32,
      TokenType::Keyword as u32,
      TokenType::Operator as u32,
      TokenType::Parameter as u32,
      TokenType::Operator as u32,
      TokenType::Type as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Type as u32,
      TokenType::Operator as u32,
      TokenType::Ident as u32,
      TokenType::Operator as u32,
      TokenType::Type as u32,
      TokenType::Operator as u32,
      TokenType::Keyword as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Property as u32,
      TokenType::Operator as u32,
      TokenType::Type as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
    ],
  );
}

#[test]
fn semantic_tokens_classify_attrs_and_visibility() {
  let (mut server, uri) =
    server_with("#[entry] pub app: fn(input: Int) -> Int = { #[cold] run; };");
  let tokens = server.semantic_tokens(&uri).data;
  let token_types = tokens
    .iter()
    .map(|token| token.token_type)
    .collect::<Vec<_>>();

  assert_eq!(
    token_types,
    vec![
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Decorator as u32,
      TokenType::Operator as u32,
      TokenType::Modifier as u32,
      TokenType::Function as u32,
      TokenType::Operator as u32,
      TokenType::Keyword as u32,
      TokenType::Operator as u32,
      TokenType::Parameter as u32,
      TokenType::Operator as u32,
      TokenType::Type as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Type as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Decorator as u32,
      TokenType::Operator as u32,
      TokenType::Ident as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
    ],
  );
}

#[test]
fn semantic_tokens_classify_using_trees() {
  let (mut server, uri) = server_with("using foo::{bar as baz};");
  let tokens = server.semantic_tokens(&uri).data;
  let token_types = tokens
    .iter()
    .map(|token| token.token_type)
    .collect::<Vec<_>>();

  assert_eq!(
    token_types,
    vec![
      TokenType::Keyword as u32,
      TokenType::Module as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Module as u32,
      TokenType::Keyword as u32,
      TokenType::Module as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
    ],
  );
}

#[test]
fn semantic_tokens_keep_labeled_expr_label() {
  let (mut server, uri) = server_with("x: = 'loop: while cond { break 'loop done };");
  let tokens = server.semantic_tokens(&uri).data;
  let token_types = tokens
    .iter()
    .map(|token| token.token_type)
    .collect::<Vec<_>>();

  assert_eq!(
    token_types,
    vec![
      TokenType::Ident as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
      TokenType::Label as u32,
      TokenType::Operator as u32,
      TokenType::Keyword as u32,
      TokenType::Ident as u32,
      TokenType::Operator as u32,
      TokenType::Keyword as u32,
      TokenType::Label as u32,
      TokenType::Ident as u32,
      TokenType::Operator as u32,
      TokenType::Operator as u32,
    ],
  );
}

#[test]
fn semantic_tokens_delta_uses_precise_dirty_range() {
  let (mut server, uri) = server_with("foo: mod;\nbar: mod;");
  let full = server.semantic_tokens(&uri);
  let previous_result_id = full.result_id.clone().unwrap();

  server.change_document_range(
    &uri,
    Range::new(Position::new(1, 2), Position::new(1, 2)),
    "z".to_string(),
  );

  let delta = server.semantic_tokens_delta(&uri, &previous_result_id);
  let SemanticTokensFullDeltaResult::TokensDelta(delta) = delta else {
    panic!("expected semantic token delta");
  };

  assert_eq!(delta.edits.len(), 1);
  assert!(delta.edits[0].start > 0);
  assert_eq!(delta.edits[0].delete_count, 10);
}

#[test]
fn semantic_tokens_delta_after_save_returns_full() {
  let (mut server, uri) = server_with("foo: mod;");
  let full = server.semantic_tokens(&uri);
  let previous_result_id = full.result_id.clone().unwrap();

  server.save_document(&uri, Some("bar: mod;".to_string()));

  let delta = server.semantic_tokens_delta(&uri, &previous_result_id);

  assert!(matches!(delta, SemanticTokensFullDeltaResult::Tokens(_)));
}

#[test]
fn semantic_tokens_delta_result_id_miss_returns_full() {
  let (mut server, uri) = server_with("foo: mod;");

  let delta = server.semantic_tokens_delta(&uri, "missing");

  assert!(matches!(delta, SemanticTokensFullDeltaResult::Tokens(_)));
}
