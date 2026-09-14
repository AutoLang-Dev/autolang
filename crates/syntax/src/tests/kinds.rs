//! Every union enum covers the kinds it models.

use super::probe::probe_red;
use crate::Red;
use crate::SyntaxKind;
use crate::ast;
use crate::ast::Node;
use crate::frag;
use crate::make::Frag;

#[test]
fn union_enums_cover_their_kinds() {
  let expr_kinds = [
    SyntaxKind::WildcardExpr,
    SyntaxKind::TupleExpr,
    SyntaxKind::ParenExpr,
    SyntaxKind::ArrayExpr,
    SyntaxKind::RepeatExpr,
    SyntaxKind::RecordExpr,
    SyntaxKind::BlockExpr,
    SyntaxKind::LiteralExpr,
    SyntaxKind::PathExpr,
    SyntaxKind::IfExpr,
    SyntaxKind::WhileExpr,
    SyntaxKind::ForExpr,
    SyntaxKind::IterateExpr,
    SyntaxKind::BinaryExpr,
    SyntaxKind::PrefixExpr,
    SyntaxKind::RefExpr,
    SyntaxKind::PostfixExpr,
    SyntaxKind::CastExpr,
    SyntaxKind::CallExpr,
    SyntaxKind::IndexExpr,
    SyntaxKind::ReturnExpr,
    SyntaxKind::BreakExpr,
    SyntaxKind::ContinueExpr,
    SyntaxKind::ClosureExpr,
    SyntaxKind::FieldExpr,
    SyntaxKind::MethodCallExpr,
    SyntaxKind::LabeledExpr,
    SyntaxKind::ChainExpr,
    SyntaxKind::ErrorExpr,
  ];
  let item_kinds = [
    SyntaxKind::FunctionItem,
    SyntaxKind::TypeItem,
    SyntaxKind::UsingItem,
    SyntaxKind::ModuleItem,
    SyntaxKind::EmptyItem,
    SyntaxKind::ErrorItem,
  ];
  let stmt_kinds = [
    SyntaxKind::LetStmt,
    SyntaxKind::ShortLetStmt,
    SyntaxKind::UsingStmt,
    SyntaxKind::ExprStmt,
    SyntaxKind::AssignStmt,
    SyntaxKind::PlaceCallStmt,
  ];
  let type_kinds = [
    SyntaxKind::InferType,
    SyntaxKind::PathType,
    SyntaxKind::FnType,
    SyntaxKind::RefType,
    SyntaxKind::PtrType,
    SyntaxKind::TupleType,
    SyntaxKind::ArrayType,
    SyntaxKind::SliceType,
    SyntaxKind::RecordType,
    SyntaxKind::ErrorType,
  ];
  let pattern_kinds = [
    SyntaxKind::WildcardPat,
    SyntaxKind::IdentPat,
    SyntaxKind::TuplePat,
    SyntaxKind::RecordPat,
    SyntaxKind::ErrorPat,
  ];
  for kind in expr_kinds {
    assert!(ast::Expr::new(probe_red(kind)).is_some(), "Expr::{kind:?}");
  }

  for kind in item_kinds {
    assert!(ast::Item::new(probe_red(kind)).is_some(), "Item::{kind:?}");
  }

  for kind in stmt_kinds {
    assert!(ast::Stmt::new(probe_red(kind)).is_some(), "Stmt::{kind:?}");
  }

  for kind in type_kinds {
    assert!(ast::Type::new(probe_red(kind)).is_some(), "Type::{kind:?}");
  }

  for kind in pattern_kinds {
    assert!(
      ast::Pattern::new(probe_red(kind)).is_some(),
      "Pattern::{kind:?} is not modeled"
    );
  }

  for kind in [
    SyntaxKind::KwType,
    SyntaxKind::KwNominal,
    SyntaxKind::KwPub,
    SyntaxKind::KwPro,
    SyntaxKind::KwPri,
  ] {
    let node = if kind == SyntaxKind::KwType || kind == SyntaxKind::KwNominal {
      SyntaxKind::TypeKind
    } else {
      SyntaxKind::Visibility
    };
    let red = Frag::node(node, [Frag::kind_token(kind)]).red();

    assert!(
      ast::TypeKind::new(red.clone()).is_some() || ast::Visibility::new(red).is_some(),
      "{kind:?} in {node:?}"
    );
  }

  assert!(
    ast::AttrArg::new(frag! { AttrArg {} }.red()).is_none(),
    "an empty AttrArg carries no value"
  );

  let expr_arg = frag! { AttrArg { LiteralExpr { Int("1") } } }.red();
  assert!(matches!(
    ast::AttrArg::new(expr_arg),
    Some(ast::AttrArg::Expr(_))
  ));

  let tree_arg = frag! {
    AttrArg { DelimitedTokenTree { OpenParen CloseParen } }
  }
  .red();
  assert!(matches!(
    ast::AttrArg::new(tree_arg),
    Some(ast::AttrArg::Tree(_))
  ));
}

#[test]
fn token_backed_types_wrap_tokens() {
  let (green, src) = frag! {
    LabeledExpr { Label("'l") Colon LiteralExpr { Int("1") } }
  }
  .finish();
  let expr = ast::LabeledExpr::new(Red::new_root(green)).expect("LabeledExpr");

  assert_eq!(&src[expr.label().expect("label").red().range()], "'l");
}
