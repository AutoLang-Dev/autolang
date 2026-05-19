use crate::{DiagPayload, Green, Indel, Red, ast::Root, build_syntax_tree, reparse};
use parser::{LexedStr, SyntaxKind, parse};
use text_size::TextRange;

fn parse_green(text: &str) -> Green {
  let lexed = LexedStr::new(text);
  let output = parse(&lexed);
  build_syntax_tree(&lexed, &output)
}

fn parse_red(text: &str) -> Red {
  Red::new_root(parse_green(text))
}

fn indel(delete: std::ops::Range<usize>, insert: &str) -> Indel {
  Indel {
    delete: TextRange::new(
      delete.start.try_into().unwrap(),
      delete.end.try_into().unwrap(),
    ),
    insert: insert.to_string(),
  }
}

fn find_error_token(tree: &Red) -> Option<Red> {
  tree
    .tokens()
    .find(|token| token.kind() == SyntaxKind::Error)
}

fn assert_reparse_matches_full(text: &str, edit: &Indel) {
  let old_tree = parse_red(text);
  let reparse = reparse(&old_tree, text, edit).expect("edit should change the tree");
  let reparsed_dump = Red::new_root(reparse.new.clone()).green().dump();
  let full_dump = parse_green(&edit.apply_to(text)).dump();

  assert_eq!(reparsed_dump, full_dump);
}

fn assert_reparse_falls_back_to_full(text: &str, edit: &Indel) {
  let old_tree = parse_red(text);
  let reparse = reparse(&old_tree, text, edit).expect("edit should change the tree");
  let reparsed_dump = Red::new_root(reparse.new.clone()).green().dump();
  let full_dump = parse_green(&edit.apply_to(text)).dump();

  assert_eq!(reparse.old.kind(), SyntaxKind::SourceFile);
  assert_eq!(reparsed_dump, full_dump);
}

#[test]
fn builds_source_file_root() {
  let tree = parse_red("foo: mod;");

  assert_eq!(tree.kind(), SyntaxKind::SourceFile);
  assert!(tree.first_child().is_some());
}

#[test]
fn records_error_token_payload() {
  let tree = parse_red("foo fn;");
  let error = find_error_token(&tree).expect("expected error token");

  assert!(matches!(error.payload().diag, Some(DiagPayload::Diag(_))));
}

#[test]
fn propagates_subtree_diagnostics_to_root() {
  let tree = parse_red("foo fn;");

  assert!(matches!(tree.payload().diag, Some(DiagPayload::Subtree)));
}

#[test]
fn reparse_returns_none_for_noop_edit() {
  let text = "foo: mod;";
  let tree = parse_red(text);
  let edit = indel(0..3, "foo");

  assert!(reparse(&tree, text, &edit).is_none());
}

#[test]
fn identifier_edit_matches_full_reparse() {
  let edit = indel(1..2, "a");

  assert_reparse_matches_full("foo: mod;", &edit);
}

#[test]
fn parameter_list_edit_matches_full_reparse() {
  let text = "add: fn(a: Int) -> Int = a;";
  let start = text.find("a: Int").unwrap();
  let edit = indel(start..start + 1, "ab");

  assert_reparse_matches_full(text, &edit);
}

#[test]
fn arg_list_edit_matches_full_reparse() {
  let text = "x: = foo(1);";
  let start = text.find(')').unwrap();
  let edit = indel(start..start, ", 2");

  assert_reparse_matches_full(text, &edit);
}

#[test]
fn array_expr_edit_matches_full_reparse() {
  let text = "x: = [1];";
  let start = text.find(']').unwrap();
  let edit = indel(start..start, ", 2");

  assert_reparse_matches_full(text, &edit);
}

#[test]
fn module_body_edit_matches_full_reparse() {
  let text = "foo: mod = { bar: mod; };";
  let start = text.find("};").unwrap();
  let edit = indel(start..start, " baz: mod;");

  assert_reparse_matches_full(text, &edit);
}

#[test]
fn dangerous_character_edit_falls_back_to_full_reparse() {
  let edit = indel(1..1, "(");

  assert_reparse_falls_back_to_full("foo: mod;", &edit);
}

#[test]
fn token_reparse_failure_falls_back_to_full_reparse() {
  let edit = indel(1..2, " a");

  assert_reparse_falls_back_to_full("foo: mod;", &edit);
}

#[test]
fn root_ast_smoke_test() {
  let tree = parse_red("#!/usr/bin/env autolang\nfoo: mod;");
  let root = Root::new(tree).expect("expected source file root");

  assert!(root.shebang().is_some());
  assert_eq!(root.items().len(), 1);
}
