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
    expr => #0 Expr;
  }
}

impl ArrayExpr {
  define_getter! {
    values => [Expr];
  }
}

impl RepeatExpr {
  define_getter! {
    value => #0 Expr;
    len => #-1 Expr;
  }
}

define_nodes! {
  ExprField: _,
}

impl ExprField {
  define_getter! {
    // `None` for positional tuple elements, which have no name.
    name => #0 Name;
    value => #-1 Expr;
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
    expr => #-1 Expr;
  }
}

// impl LiteralExpr {}

impl PathExpr {
  define_getter! {
    path => #0 Path;
  }
}

define_nodes! {
  ElseClause: _,
}

impl IfExpr {
  define_getter! {
    cond => #0 Expr;
    then => #1 BlockExpr;
    else_branch => #-1 ElseClause;
  }
}

impl WhileExpr {
  define_getter! {
    cond => #0 Expr;
    then => #1 BlockExpr;
    else_branch => #-1 ElseClause;
  }
}

impl ForExpr {
  define_getter! {
    pat => #0 Pattern;
    range => #1 Expr;
    then => #2 BlockExpr;
    else_branch => #-1 ElseClause;
  }
}

impl IterateExpr {
  define_getter! {
    pat => #0 Pattern;
    init => #1 Expr;
    body => #-1 BlockExpr;
  }
}

// impl BinaryExpr {}

// impl PrefixExpr {}

impl RefExpr {
  define_getter! {
    mutable => ? KwMut;
    expr => #-1 Expr;
  }
}

// impl PostfixExpr {}

impl CastExpr {
  define_getter! {
    expr => #0 Expr;
    ty => #-1 Type;
  }
}

define_nodes! {
  ArgList: _,
  IndexArg: _,
}

impl ArgList {
  define_getter! {
    args => #0 Expr;
  }
}

impl CallExpr {
  define_getter! {
    callee => #0 Expr;
    args => #-1 ArgList;
  }
}

impl IndexArg {
  define_getter! {
    index => #0 Expr;
  }
}

impl IndexExpr {
  define_getter! {
    expr => #0 Expr;
    index => #-1 IndexArg;
  }
}

impl ReturnExpr {
  define_getter! {
    expr => #-1 Expr;
  }
}

define_token_nodes! {
  Label: _,
}

impl BreakExpr {
  define_getter! {
    label => Label;
    expr => #-1 Expr;
  }
}

impl ContinueExpr {
  define_getter! {
    label => Label;
    expr => #-1 Expr;
  }
}

// impl ClosureExpr {}

impl FieldExpr {
  define_getter! {
    expr => #0 Expr;
    field => #-1 Name;
  }
}

// impl MethodCallExpr {}

impl LabeledExpr {
  define_getter! {
    label => Label;
    expr => #-1 Expr;
  }
}

// impl ChainExpr {}

// impl ErrorExpr {}
