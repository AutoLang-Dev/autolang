use crate::{
  Red,
  ast::{Expr, Item, Node},
};
use parser::T;

define_node_enum! {
  Stmt {
    Item(Item),
    Expr(ExprStmt),
    Assign(AssignStmt),
  } no_new
}

impl Stmt {
  pub fn new(red: Red) -> Option<Self> {
    Some(match red.kind() {
      ExprStmt::KIND => Self::Expr(ExprStmt::new(red)?),
      AssignStmt::KIND => Self::Assign(AssignStmt::new(red)?),
      _ => Self::Item(Item::new(red)?),
    })
  }
}

define_nodes! {
  ExprStmt: _,
  AssignStmt: _,
}

define_attr! {
  Stmt,
  ExprStmt,
  AssignStmt,
}

impl ExprStmt {
  define_getter! {
    expr => ! Expr;
  }
}

impl AssignStmt {
  define_getter! {
    lhs => ! Expr;
    op => ! AssignOp;
    rhs <= ! Expr;
  }
}

pub struct AssignOp {
  red: Red,
}

impl AssignOp {
  pub fn new(red: Red) -> Option<Self> {
    if !matches!(
      red.kind(),
      T![=] | T![+=] | T![-=] | T![*=] | T![/=] | T![%=] | T![<<=] | T![>>=]
    ) {
      return None;
    }
    Some(Self { red })
  }
}

impl Node for AssignOp {
  fn red(&self) -> &Red {
    &self.red
  }
}
