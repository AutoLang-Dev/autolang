//! Accessors that pick a child by position, checked on the shapes they disambiguate.

use crate::Red;
use crate::ast;
use crate::ast::Node;
use crate::frag;

#[test]
fn fn_type_ret_is_the_return_type() {
  let (green, src) = frag! {
    FnType {
      TupleType {
        OpenParen
        TypeField {
          Name { Ident("next") }
          Colon
          PathType { Path { PathSegment { Name { Ident("Int") } } } }
        }
        CloseParen
      }
      ThinArrow
      PathType { Path { PathSegment { Name { Ident("Out") } } } }
    }
  }
  .finish();

  let ty = ast::FnType::new(Red::new_root(green)).expect("FnType");

  assert_eq!(
    &src[ty.params().expect("params").red().range()],
    "(next:Int)"
  );
  assert_eq!(&src[ty.ret().expect("ret").red().range()], "Out");
}

#[test]
fn while_body_is_not_the_head_block() {
  let (green, src) = frag! {
    WhileExpr {
      BlockExpr { OpenBrace Int("1") CloseBrace }
      BlockExpr { OpenBrace Int("2") CloseBrace }
    }
  }
  .finish();
  let expr = ast::WhileExpr::new(Red::new_root(green)).expect("WhileExpr");

  assert_eq!(&src[expr.cond().expect("cond").red().range()], "{1}");
  assert_eq!(&src[expr.then().expect("then").red().range()], "{2}");
}

#[test]
fn for_body_is_not_the_iterable_block() {
  let (green, src) = frag! {
    ForExpr {
      IdentPat { Name { Ident("i") } }
      KwIn
      BlockExpr { OpenBrace Int("1") CloseBrace }
      BlockExpr { OpenBrace Int("2") CloseBrace }
    }
  }
  .finish();
  let expr = ast::ForExpr::new(Red::new_root(green)).expect("ForExpr");

  assert_eq!(&src[expr.range().expect("range").red().range()], "{1}");
  assert_eq!(&src[expr.then().expect("then").red().range()], "{2}");
}

#[test]
fn iterate_body_is_not_the_init_block() {
  let (green, src) = frag! {
    IterateExpr {
      IdentPat { Name { Ident("i") } }
      ColonEq
      BlockExpr { OpenBrace Int("1") CloseBrace }
      BlockExpr { OpenBrace Int("2") CloseBrace }
    }
  }
  .finish();
  let expr = ast::IterateExpr::new(Red::new_root(green)).expect("IterateExpr");

  assert_eq!(&src[expr.init().expect("init").red().range()], "{1}");
  assert_eq!(&src[expr.body().expect("body").red().range()], "{2}");
}

#[test]
fn positional_pat_field_has_no_name() {
  let (green, src) = frag! {
    PatField { IdentPat { Name { Ident("a") } } }
  }
  .finish();
  let field = ast::PatField::new(Red::new_root(green)).expect("PatField");

  assert!(field.name().is_none(), "positional fields carry no name");

  let (green, src_named) = frag! {
    PatField { Name { Ident("a") } IdentPat { Name { Ident("b") } } }
  }
  .finish();
  let named = ast::PatField::new(Red::new_root(green)).expect("PatField");
  let name = named.name().expect("named field");

  assert_eq!(&src_named[name.red().range()], "a");
  assert_ne!(src_named.len(), src.len());
}

#[test]
fn ident_pat_exposes_its_name() {
  let (green, src) = frag! { IdentPat { Name { Ident("x") } } }.finish();
  let pat = ast::IdentPat::new(Red::new_root(green)).expect("IdentPat");

  assert_eq!(&src[pat.name().expect("name").red().range()], "x");
}
