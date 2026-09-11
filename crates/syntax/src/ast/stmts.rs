use crate::{
  Red,
  ast::{Expr, Name, Node, Pattern, Type},
};
use parser::T;

define_node_enum! {
  Stmt {
    Let(LetStmt),
    ShortLet(ShortLetStmt),
    Using(UsingStmt),
    Expr(ExprStmt),
    Assign(AssignStmt),
    PlaceCall(PlaceCallStmt),
  } no_new
}

impl Stmt {
  pub fn new(red: Red) -> Option<Self> {
    Some(match red.kind() {
      LetStmt::KIND => Self::Let(LetStmt::new(red)?),
      ShortLetStmt::KIND => Self::ShortLet(ShortLetStmt::new(red)?),
      UsingStmt::KIND => Self::Using(UsingStmt::new(red)?),
      ExprStmt::KIND => Self::Expr(ExprStmt::new(red)?),
      AssignStmt::KIND => Self::Assign(AssignStmt::new(red)?),
      PlaceCallStmt::KIND => Self::PlaceCall(PlaceCallStmt::new(red)?),
      _ => return None,
    })
  }
}

define_nodes! {
  LetStmt: _,
  ShortLetStmt: _,
  UsingStmt: _,
  ExprStmt: _,
  AssignStmt: _,
  PlaceCallStmt: _,
}

define_attr! {
  Stmt,
  LetStmt,
  ShortLetStmt,
  UsingStmt,
  ExprStmt,
  AssignStmt,
  PlaceCallStmt,
}

impl LetStmt {
  define_getter! {
    pat => Pattern;
    ty => Type;
    init <= Expr;
  }
}

impl ShortLetStmt {
  define_getter! {
    name => Name;
    init <= Expr;
  }
}

impl ExprStmt {
  define_getter! {
    expr => Expr;
  }
}

impl AssignStmt {
  define_getter! {
    lhs => Expr;
    op => AssignOp;
    rhs <= Expr;
  }
}

impl PlaceCallStmt {
  define_getter! {
    // The expression handed the destination, e.g. `f()` in `f() in p;`.
    subject => Expr;
    // The destination; semantically a reference or pointer. It is the last
    // expression, since the subject comes first.
    place <= Expr;
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
