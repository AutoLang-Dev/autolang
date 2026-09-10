//! Self-tests for the `make` construction layer.
//!
//! These check that constructed fragments keep text and tree in sync, that the
//! `frag!` DSL composes, and that hand-built shapes agree with what the parser
//! produces (spot checks).

use super::parse_red;
use crate::{
  Green, Red,
  SyntaxKind::{self, *},
  ast, frag,
  make::{Frag, joined},
};
use std::string::String;

fn kinds_without_trivia(red: &Red) -> Vec<SyntaxKind> {
  red
    .descendants()
    .map(|node| node.kind())
    .filter(|kind| !kind.is_trivia())
    .collect()
}

fn assert_text_matches_width(frag: Frag) -> (Green, String) {
  let text = frag.text().to_owned();
  let (green, src) = frag.finish();

  assert_eq!(src, text, "text survives finish()");
  assert_eq!(
    u32::from(green.width()),
    src.len() as u32,
    "tree width matches text length for {src:?}"
  );

  (green, src)
}

fn path_expr(segment: &str) -> Frag {
  frag! { PathExpr { Path { PathSegment { Name { Ident(segment) } } } } }
}

fn path_ty(segment: &str) -> Frag {
  frag! { PathType { Path { PathSegment { Name { Ident(segment) } } } } }
}

fn unit_fn_ty() -> Frag {
  frag! { FnType { TupleType { OpenParen CloseParen } ThinArrow TupleType { OpenParen CloseParen } } }
}

#[test]
fn fragments_keep_text_and_tree_in_sync() {
  let frags = [
    frag! { Name { Ident("x") } },
    frag! { Path { PathSegment { Name { Ident("a") } } ColonColon PathSegment { Name { Ident("b") } } } },
    path_ty("Int"),
    path_expr("x"),
    frag! { LiteralExpr { Int("114514") } },
    frag! { LiteralExpr { KwTrue } },
    unit_fn_ty(),
    frag! { RefExpr { Amp KwMut @(path_expr("x")) } },
    frag! { BinaryExpr { @(path_expr("x")) Plus LiteralExpr { Int("1") } } },
    frag! {
      CallExpr {
        @(path_expr("f"))
        ArgList { TupleExpr { OpenParen ExprField { LiteralExpr { Int("1") } } CloseParen } }
      }
    },
    frag! { BlockExpr { OpenBrace LiteralExpr { Int("1") } CloseBrace } },
    frag! { ClosureExpr { Backslash Dot BlockExpr { OpenBrace CloseBrace } } },
    frag! { LetStmt { KwLet WildcardPat { Underscore } Semi } },
    frag! { FunctionItem { Name { Ident("f") } Colon @(unit_fn_ty()) Semi } },
    frag! { SourceFile { ModuleInner {} } },
    frag! { TupleType { OpenParen @(path_ty("Int")) CloseParen } },
  ];

  for frag in frags {
    assert_text_matches_width(frag);
  }
}

#[test]
fn splices_fragments_and_runtime_lists() {
  let fields = ["a", "b"]
    .into_iter()
    .map(|name| frag! { TypeField { Name { Ident(name) } Colon @(path_ty("Int")) } })
    .collect::<Vec<_>>();

  let (green, src) = frag! {
    TupleType { OpenParen @(joined(Comma, fields)) CloseParen }
  }
  .finish();

  assert_eq!(src, "(a:Int,b:Int)");
  assert_eq!(
    kinds_without_trivia(&Red::new_root(green)),
    [
      TupleType,
      OpenParen,
      TypeField,
      Name,
      Ident,
      Colon,
      PathType,
      Path,
      PathSegment,
      Name,
      Ident,
      Comma,
      TypeField,
      Name,
      Ident,
      Colon,
      PathType,
      Path,
      PathSegment,
      Name,
      Ident,
      CloseParen,
    ]
  );
}

#[test]
fn ranges_slice_the_text_they_were_built_from() {
  let params = joined(
    Comma,
    [frag! { TypeField { Name { Ident("a") } Colon @(path_ty("Int")) } }],
  );
  let (green, src) = frag! {
    FunctionItem {
      Name { Ident("add") }
      Colon
      FnType { TupleType { OpenParen @(params) CloseParen } ThinArrow @(path_ty("Int")) }
      Semi
    }
  }
  .finish();

  let red = Red::new_root(green.clone());
  assert_eq!(&src[red.range()], "add:(a:Int)->Int;");
  assert!(ast::FunctionItem::new(red).is_some());
}

#[test]
fn dsl_builds_the_expected_kind_sequence() {
  let (green, _src) = frag! {
    SourceFile {
      ModuleInner {
        FunctionItem {
          Name { Ident("main") }
          Colon
          @(unit_fn_ty())
          Eq
          ClosureExpr {
            Backslash
            Dot
            BlockExpr { OpenBrace ShortLetStmt { Name { Ident("x") } ColonEq LiteralExpr { Int("1") } Semi } CloseBrace }
          }
          Semi
        }
      }
    }
  }
  .finish();

  let got = kinds_without_trivia(&Red::new_root(green));
  let want = vec![
    SourceFile,
    ModuleInner,
    FunctionItem,
    Name,
    Ident,
    Colon,
    FnType,
    TupleType,
    OpenParen,
    CloseParen,
    ThinArrow,
    TupleType,
    OpenParen,
    CloseParen,
    Eq,
    ClosureExpr,
    Backslash,
    Dot,
    BlockExpr,
    OpenBrace,
    ShortLetStmt,
    Name,
    Ident,
    ColonEq,
    LiteralExpr,
    Int,
    Semi,
    CloseBrace,
    Semi,
  ];

  assert_eq!(got, want);
}

/// Spot check: the parser and a hand-built tree agree on the shape (trivia aside).
fn assert_matches_parser(source: &str, built: Frag) {
  let (green, _) = built.finish();
  let from_parser = parse_red(source);
  let from_make = Red::new_root(green);

  assert_eq!(
    kinds_without_trivia(&from_make),
    kinds_without_trivia(&from_parser),
    "the hand-built tree disagrees with the parser for {source:?}"
  );
}

fn file(items: impl IntoIterator<Item = Frag>) -> Frag {
  frag! { SourceFile { ModuleInner { @(items) } } }
}

fn unit_fn(body: Frag) -> Frag {
  file([frag! {
    FunctionItem {
      Name { Ident("main") }
      Colon
      @(unit_fn_ty())
      Eq
      ClosureExpr { Backslash Dot BlockExpr { OpenBrace @(body) CloseBrace } }
      Semi
    }
  }])
}

#[test]
fn make_agrees_with_the_parser_on_a_function_item() {
  assert_matches_parser(
    "main: () -> () = \\.{ x := 1; };",
    unit_fn(frag! {
      ShortLetStmt { Name { Ident("x") } ColonEq LiteralExpr { Int("1") } Semi }
    }),
  );
}

#[test]
fn make_agrees_with_the_parser_on_a_tuple_pattern_let() {
  let fields = joined(
    Comma,
    [
      frag! { PatField { IdentPat { Name { Ident("a") } } } },
      frag! { PatField { IdentPat { Name { Ident("b") } } } },
    ],
  );
  let values = ["1", "2"]
    .into_iter()
    .map(|value| frag! { ExprField { LiteralExpr { Int(value) } } });

  assert_matches_parser(
    "main: () -> () = \\.{ let (a, b) = (1, 2); };",
    unit_fn(frag! {
      LetStmt {
        KwLet
        TuplePat { OpenParen @(fields) CloseParen }
        Eq
        TupleExpr { OpenParen @(joined(Comma, values)) CloseParen }
        Semi
      }
    }),
  );
}

#[test]
fn make_agrees_with_the_parser_on_a_call() {
  assert_matches_parser(
    "main: () -> () = \\.{ f(1); };",
    unit_fn(frag! {
      ExprStmt {
        CallExpr {
          @(path_expr("f"))
          ArgList { TupleExpr { OpenParen ExprField { LiteralExpr { Int("1") } } CloseParen } }
        }
        Semi
      }
    }),
  );
}
