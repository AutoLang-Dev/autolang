mod reparse;

use crate::{Indel, Lang, Parse, Reparse, SyntaxKind, TextRange, TextSize, debug, parse};
use parser::T;
use rowan::Language;

macro_rules! snap {
  ($desc:expr, $value:expr $(,)?) => {{
    insta::with_settings!({
      description => $desc,
    }, {
      insta::assert_debug_snapshot!($value);
    });
  }};
}

macro_rules! text_snap {
  ($desc:expr, $value:expr $(,)?) => {{
    insta::with_settings!({
      description => $desc,
    }, {
      insta::assert_snapshot!($value);
    });
  }};
}

fn dump_parse(parse: &Parse) -> Vec<String> {
  debug::syntax_tree_lines(parse)
}

fn dump(input: &str) -> Vec<String> {
  dump_parse(&parse(input))
}

fn reparse_eq_full(old: &str, delete: std::ops::Range<u32>, insert: &str) -> crate::Reparse {
  let old_parse = parse(old);
  let indel = Indel {
    delete: TextRange::new(TextSize::from(delete.start), TextSize::from(delete.end)),
    insert: insert.to_string(),
  };
  let mut new_text = old.to_string();
  indel.apply(&mut new_text);
  let reparse = old_parse.reparse(old, &indel);
  let full = parse(&new_text);
  let reparse_parse = reparse_parse(&reparse).expect("reparse should produce a parse");

  assert_eq!(dump_parse(reparse_parse), dump_parse(&full));
  assert_eq!(reparse_parse.errors(), full.errors());
  reparse
}

fn reparse_parse(reparse: &Reparse) -> Option<&Parse> {
  match reparse {
    Reparse::Noop => None,
    Reparse::Token(reparse) => Some(&reparse.parse),
    Reparse::Node(reparse) => Some(&reparse.parse),
    Reparse::Full(parse) => Some(parse),
  }
}

#[test]
fn mod_decl_tree() {
  let input = "foo: mod;";

  snap!(input, dump(input));
}

#[test]
fn trivia_tree() {
  let input = "#!/usr/bin/env autolang\nfoo : mod; // keep me";

  snap!(input, dump(input));
}

#[test]
fn item_layer_tree() {
  let input = "x: Int = 42; add: fn(a: Int) -> Int = a; Point: type = { x: Int };";

  snap!(input, dump(input));
}

#[test]
fn chain_expr_tree() {
  let input = "test: = 1 && 2 && 3;";

  text_snap!(input, debug::syntax_tree(&parse(input)));
}

#[test]
fn unexpected_item_errors() {
  let input = "foo fn;";
  let parse = parse(input);

  snap!(input, dump(input));
  snap!("errors", parse.errors());
}

#[test]
fn debug_syntax_tree_includes_errors() {
  let input = "foo fn;";
  let parse = parse(input);

  text_snap!(input, debug::syntax_tree(&parse));
}

#[test]
fn syntax_kind_roundtrip() {
  for kind in [
    SyntaxKind::SourceFile,
    SyntaxKind::Whitespace,
    SyntaxKind::Ident,
    T![->],
    SyntaxKind::ModuleItem,
  ] {
    let raw = Lang::kind_to_raw(kind);
    assert_eq!(Lang::kind_from_raw(raw), kind);
  }
}
