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
