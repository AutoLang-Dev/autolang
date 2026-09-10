use crate::ast::{Name, Path, Pattern, Stmt, Type};

define_node_enum! {
  Expr {
    Wildcard(WildcardExpr),
    Tuple(TupleExpr),
    Paren(ParenExpr),
    Array(ArrayExpr),
    Repeat(RepeatExpr),
    Record(RecordExpr),
    Block(BlockExpr),
    Literal(LiteralExpr),
    Path(PathExpr),
    If(IfExpr),
    While(WhileExpr),
    For(ForExpr),
    Iterate(IterateExpr),
    Binary(BinaryExpr),
    Prefix(PrefixExpr),
    Ref(RefExpr),
    Postfix(PostfixExpr),
    Cast(CastExpr),
    Call(CallExpr),
    Index(IndexExpr),
    Return(ReturnExpr),
    Break(BreakExpr),
    Continue(ContinueExpr),
    Closure(ClosureExpr),
    Field(FieldExpr),
    MethodCall(MethodCallExpr),
    Labeled(LabeledExpr),
    Chain(ChainExpr),
    Error(ErrorExpr),
  }
}

define_nodes! {
  WildcardExpr: _,
  TupleExpr: _,
  ParenExpr: _,
  ArrayExpr: _,
  RepeatExpr: _,
  RecordExpr: _,
  BlockExpr: _,
  LiteralExpr: _,
  PathExpr: _,
  IfExpr: _,
  WhileExpr: _,
  ForExpr: _,
  IterateExpr: _,
  BinaryExpr: _,
  PrefixExpr: _,
  RefExpr: _,
  PostfixExpr: _,
  CastExpr: _,
  CallExpr: _,
  IndexExpr: _,
  ReturnExpr: _,
  BreakExpr: _,
  ContinueExpr: _,
  ClosureExpr: _,
  FieldExpr: _,
  MethodCallExpr: _,
  LabeledExpr: _,
  ChainExpr: _,
  ErrorExpr: _,
}

impl TupleExpr {
  define_getter! {
    fields => [ExprField];
  }
}

impl ParenExpr {
  define_getter! {
    expr => Expr;
  }
}

impl ArrayExpr {
  define_getter! {
    values => [Expr];
  }
}

impl RepeatExpr {
  define_getter! {
    value => ! Expr;
    len <= ! Expr;
  }
}

define_nodes! {
  ExprField: _,
}

impl ExprField {
  define_getter! {
    // `None` for positional tuple elements, which have no name.
    name => Name;
    value <= Expr;
  }
}

impl RecordExpr {
  define_getter! {
    fields => [ExprField];
  }
}

impl BlockExpr {
  define_getter! {
    stmts => [Stmt];
    expr <= Expr;
  }
}

// impl LiteralExpr {}

impl PathExpr {
  define_getter! {
    path => ! Path;
  }
}

define_nodes! {
  ElseClause: _,
}

impl IfExpr {
  define_getter! {
    cond => ! Expr;
    then <= ! BlockExpr;
    else_branch <= ElseClause;
  }
}

impl WhileExpr {
  define_getter! {
    cond => ! Expr;
    then => ! BlockExpr;
    else_branch <= ElseClause;
  }
}

impl ForExpr {
  define_getter! {
    pat => ! Pattern;
    range => ! Expr;
    then => ! BlockExpr;
    else_branch <= ElseClause;
  }
}

impl IterateExpr {
  define_getter! {
    pat => ! Pattern;
    init => ! Expr;
    body => ! BlockExpr;
  }
}

// impl BinaryExpr {}

// impl PrefixExpr {}

impl RefExpr {
  define_getter! {
    mutable => ? KwMut;
    expr <= ! Expr;
  }
}

// impl PostfixExpr {}

impl CastExpr {
  define_getter! {
    expr => ! Expr;
    ty <= ! Type;
  }
}

define_nodes! {
  ArgList: _,
  IndexArg: _,
}

impl ArgList {
  define_getter! {
    args => ! Expr;
  }
}

impl CallExpr {
  define_getter! {
    callee => ! Expr;
    args => ArgList;
  }
}

impl IndexArg {
  define_getter! {
    index => ! Expr;
  }
}

impl IndexExpr {
  define_getter! {
    expr => ! Expr;
    index <= ! IndexArg;
  }
}

impl ReturnExpr {
  define_getter! {
    expr <= Expr;
  }
}

define_nodes! {
  Label: _,
}

impl BreakExpr {
  define_getter! {
    label => Label;
    expr <= Expr;
  }
}

impl ContinueExpr {
  define_getter! {
    label => Label;
    expr <= Expr;
  }
}

// impl ClosureExpr {}

impl FieldExpr {
  define_getter! {
    expr => ! Expr;
    // `None` when the access is not a name (e.g. the rejected `tup.0`).
    field => Name;
  }
}

// impl MethodCallExpr {}

impl LabeledExpr {
  define_getter! {
    label => ! Label;
    expr <= ! Expr;
  }
}

// impl ChainExpr {}

// impl ErrorExpr {}
