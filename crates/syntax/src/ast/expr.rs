use crate::ast::{Name, Path, Pattern, Stmt, Type};

define_node_enum! {
  Expr {
    Wildcard(WildcardExpr),
    Tuple(TupleExpr),
    Paren(ParenExpr),
    Array(ArrayExpr),
    Repeat(RepeatExpr),
    Struct(StructExpr),
    Block(BlockExpr),
    Literal(LiteralExpr),
    Path(PathExpr),
    Case(CaseExpr),
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
  StructExpr: _,
  BlockExpr: _,
  LiteralExpr: _,
  PathExpr: _,
  CaseExpr: _,
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
    fields => [Expr];
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
  FieldName: _,
  FieldValue: _,
}

impl FieldValue {
  define_getter! {
    name => ! FieldName;
    value <= Expr;
  }
}

impl StructExpr {
  define_getter! {
    fields => [FieldValue];
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

// impl CaseExpr {}

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
    cond => ! Expr;
    then => ! BlockExpr;
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
    args => [Expr];
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
    field => ! Name;
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
