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

  assert_eq!(&src[ty.params().red().range()], "(next:Int)");
  assert_eq!(&src[ty.ret().red().range()], "Out");
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

  assert_eq!(&src[expr.cond().red().range()], "{1}");
  assert_eq!(&src[expr.then().red().range()], "{2}");
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

  assert_eq!(&src[expr.range().red().range()], "{1}");
  assert_eq!(&src[expr.then().red().range()], "{2}");
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

  assert_eq!(&src[expr.init().red().range()], "{1}");
  assert_eq!(&src[expr.body().red().range()], "{2}");
}
