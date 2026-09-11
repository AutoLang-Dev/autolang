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

#[test]
fn positional_accessors_use_the_node_index() {
  let (green, src) = frag! {
    RepeatExpr { OpenBrack LiteralExpr { Int("1") } Semi LiteralExpr { Int("2") } CloseBrack }
  }
  .finish();
  let expr = ast::RepeatExpr::new(Red::new_root(green)).expect("RepeatExpr");

  assert_eq!(&src[expr.value().expect("value").red().range()], "1");
  assert_eq!(&src[expr.len().expect("len").red().range()], "2");

  let (green, src) = frag! {
    AssignStmt { PathExpr { Path { PathSegment { Name { Ident("a") } } } } PlusEq LiteralExpr { Int("1") } Semi }
  }
  .finish();
  let stmt = ast::Stmt::new(Red::new_root(green)).expect("Stmt");
  let ast::Stmt::Assign(stmt) = stmt else {
    panic!("expected an assign statement");
  };

  assert_eq!(&src[stmt.lhs().expect("lhs").red().range()], "a");
  assert_eq!(&src[stmt.rhs().expect("rhs").red().range()], "1");
}

#[test]
fn node_children_index_from_either_end() {
  let (green, src) = frag! {
    IterateExpr {
      IdentPat { Name { Ident("i") } }
      ColonEq
      LiteralExpr { Int("1") }
      BlockExpr { OpenBrace CloseBrace }
    }
  }
  .finish();
  let red = Red::new_root(green);
  let text = |child: Red| src[child.range()].to_owned();

  assert_eq!(text(ast::node_children(&red).next().unwrap()), "i");
  assert_eq!(text(ast::node_child_from_end(&red, 1).unwrap()), "{}");
  assert_eq!(text(ast::node_child_from_end(&red, 2).unwrap()), "1");
  assert_eq!(text(ast::node_child_from_end(&red, 3).unwrap()), "i");
  assert!(ast::node_child_from_end(&red, 4).is_none());
}

#[test]
fn block_tail_is_the_last_child() {
  let (green, src) = frag! {
    BlockExpr {
      OpenBrace
      ExprStmt { LiteralExpr { Int("1") } Semi }
      ExprStmt { LiteralExpr { Int("2") } Semi }
      LiteralExpr { Int("3") }
      CloseBrace
    }
  }
  .finish();
  let block = ast::BlockExpr::new(Red::new_root(green)).expect("BlockExpr");

  assert_eq!(block.stmts().len(), 2);
  assert_eq!(&src[block.expr().expect("tail").red().range()], "3");
}

#[test]
fn item_body_follows_its_attributes() {
  let (green, src) = frag! {
    FunctionItem {
      Attr { Hash AttrInner { OpenBrack AttrItem { Path { PathSegment { Name { Ident("a") } } } } CloseBrack } }
      Name { Ident("f") }
      Colon
      FnType { TupleType { OpenParen CloseParen } ThinArrow TupleType { OpenParen CloseParen } }
      Eq
      PathExpr { Path { PathSegment { Name { Ident("body") } } } }
      Semi
    }
  }
  .finish();
  let item = ast::FunctionItem::new(Red::new_root(green)).expect("FunctionItem");

  assert_eq!(&src[item.attr().expect("attr").red().range()], "#[a]");
  assert_eq!(&src[item.body().expect("body").red().range()], "body");
}

#[test]
fn attributes_do_not_shift_statement_operands() {
  let (green, src) = frag! {
    AssignStmt {
      Attr { Hash AttrInner { OpenBrack AttrItem { Path { PathSegment { Name { Ident("a") } } } } CloseBrack } }
      PathExpr { Path { PathSegment { Name { Ident("x") } } } }
      PlusEq
      LiteralExpr { Int("1") }
      Semi
    }
  }
  .finish();
  let stmt = ast::Stmt::new(Red::new_root(green)).expect("Stmt");
  let ast::Stmt::Assign(stmt) = stmt else {
    panic!("expected an assign statement");
  };

  assert_eq!(&src[stmt.attr().expect("attr").red().range()], "#[a]");
  assert_eq!(&src[stmt.lhs().expect("lhs").red().range()], "x");
  assert_eq!(&src[stmt.rhs().expect("rhs").red().range()], "1");
}
