//! Local reparse against a full reparse of the edited text.

use super::{parse_green, parse_red};
use crate::{Indel, Red, SyntaxKind, reparse};
use text_size::TextRange;

fn indel(delete: std::ops::Range<usize>, insert: &str) -> Indel {
  Indel {
    delete: TextRange::new(
      delete.start.try_into().unwrap(),
      delete.end.try_into().unwrap(),
    ),
    insert: insert.to_string(),
  }
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
