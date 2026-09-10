//! Building a tree, and the diagnostics carried on it.

use super::parse_red;
use crate::{DiagPayload, Red, SyntaxKind, ast::Root};

fn find_error_token(tree: &Red) -> Option<Red> {
  tree
    .tokens()
    .find(|token| token.kind() == SyntaxKind::Error)
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
fn root_ast_smoke_test() {
  let tree = parse_red("#!/usr/bin/env autolang\nfoo: mod;");
  let root = Root::new(tree).expect("expected source file root");

  assert_eq!(root.items().len(), 1);
}
