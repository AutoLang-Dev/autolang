//! The accessors each node kind exposes.

use crate::ast;
use crate::frag;

#[test]
fn tuple_and_record_patterns_are_reachable() {
  let tuple = frag! {
    LetStmt {
      KwLet
      TuplePat {
        OpenParen
        PatField { IdentPat { Name { Ident("a") } } }
        Comma
        PatField { IdentPat { Name { Ident("b") } } }
        CloseParen
      }
      Semi
    }
  }
  .red();
  let let_stmt = ast::LetStmt::new(tuple).expect("LetStmt");

  assert!(
    matches!(let_stmt.pat(), Some(ast::Pattern::Tuple(_))),
    "`let (a, b)` must model its tuple pattern"
  );

  let record = frag! {
    RecordPat { OpenBrace PatField { Name { Ident("a") } } CloseBrace }
  }
  .red();

  assert!(matches!(
    ast::Pattern::new(record),
    Some(ast::Pattern::Record(_))
  ));
}

#[test]
fn ast_nodes_clone_compare_and_hash() {
  let red = frag! { Name { Ident("x") } }.red();
  let name = ast::Name::new(red.clone()).expect("Name");
  let same = ast::Name::new(red).expect("Name");
  let other = ast::Name::new(frag! { Name { Ident("y") } }.red()).expect("Name");

  assert_eq!(name.clone(), name);
  assert_eq!(name, same);
  assert_ne!(name, other);
  assert_eq!(format!("{name:?}"), "Name@0..1");

  let mut set = std::collections::HashSet::new();
  set.insert(name.clone());
  assert!(set.contains(&same));
  assert!(!set.contains(&other));

  let expr = ast::Expr::new(frag! { LiteralExpr { Int("1") } }.red()).expect("Expr");
  assert_eq!(expr.clone(), expr);
  assert!(format!("{expr:?}").starts_with("Literal("));
}
