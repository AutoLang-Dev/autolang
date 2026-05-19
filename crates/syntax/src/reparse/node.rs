use crate::{
  Green, Indel, Red,
  ast::{ForExpr, IfExpr, IterateExpr, Node, WhileExpr},
  build_syntax_tree,
};
use parser::{Input, LexedStr, Output, Parser, SyntaxKind::*, grammar::*};
use text_size::{TextRange, TextSize};

pub fn reparse(node: &mut Red, source: &str, indel: &Indel) -> Option<Green> {
  let (candidate, reparser) = node
    .ancestors()
    .find_map(|c| Reparser::new(&c, indel.delete).map(|x| (c, x)))?;

  let rel_del = indel.delete - candidate.range().start();
  let rel_indel = Indel {
    delete: rel_del,
    insert: indel.insert.clone(),
  };

  let text = rel_indel.apply_to(&source[candidate.range()]);
  let lexed = LexedStr::new(&text);
  let output = reparser.parse(lexed.to_input());

  *node = candidate;
  Some(build_syntax_tree(&lexed, &output))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reparser {
  ArgList,
  ArrayExpr,
  ArrayOrSliceType,
  AttrInner,
  BlockExpr,
  BraceExpr,
  DelimitedTokenTree,
  IndexArg,
  Module,
  ParameterList,
  StructType,
  TupleOrParenExpr,
  // Disabled for now: `tuple_or_paren_type` also parses fn-ptr params and
  // finalizes the same prefix as `ParenType`, `TupleType`, or `FnPtrType`
  // depending on trailing lookahead. Incremental reparse can therefore mutate
  // an existing fn-ptr param list into a paren/tuple type. Re-enable this only
  // after the type parser is refactored so the reparse entry point has a stable
  // node shape and does not rely on this coupled completion logic.
  UsingTreeList,
}

impl Reparser {
  pub fn new(node: &Red, delete: TextRange) -> Option<Self> {
    let reparser = match node.kind() {
      ArgList => Self::ArgList,
      ArrayExpr | RepeatExpr => Self::ArrayExpr,
      ArrayType | SliceType => Self::ArrayOrSliceType,
      AttrInner => Self::AttrInner,
      BlockExpr if must_be_block(node) => Self::BlockExpr,
      BlockExpr | StructExpr => Self::BraceExpr,
      DelimitedTokenTree => Self::DelimitedTokenTree,
      IndexArg => Self::IndexArg,
      Module => Self::Module,
      ParameterList => Self::ParameterList,
      StructType => Self::StructType,
      ParenExpr | TupleExpr => Self::TupleOrParenExpr,
      UsingTreeList => Self::UsingTreeList,
      _ => return None,
    };

    let inner_range = {
      let range = node.range();
      let one = TextSize::new(1);
      TextRange::new(range.start() + one, range.end() - one)
    };
    if !inner_range.contains_range(delete) {
      return None;
    }

    Some(reparser)
  }

  pub fn parse(self, input: Input) -> Output {
    let mut parser = Parser::new(input);

    let parse = match self {
      Self::ArgList => expr::arg_list,
      Self::ArrayExpr => expr::array_expr,
      Self::ArrayOrSliceType => types::array_or_slice_type,
      Self::AttrInner => attrs::attr_inner,
      Self::BlockExpr => expr::block_expr,
      Self::BraceExpr => expr::brace_expr,
      Self::DelimitedTokenTree => token_trees::delimited_token_tree,
      Self::IndexArg => expr::index_arg,
      Self::Module => items::module,
      Self::ParameterList => func::parameter_list,
      Self::StructType => types::struct_type,
      Self::TupleOrParenExpr => expr::tuple_or_paren_expr,
      Self::UsingTreeList => items::using_tree_list,
    };

    parse(&mut parser);
    parser.finish()
  }
}

fn must_be_block(node: &Red) -> bool {
  let Some(parent) = node.parent() else {
    return false;
  };

  let node = node.green();

  macro_rules! helper {
    ($t:ty, $f:ident) => {
      <$t>::new(parent).unwrap().$f().green() != node
    };
  }

  match parent.kind() {
    LabeledExpr => true,
    IfExpr => helper!(IfExpr, cond),
    WhileExpr => helper!(WhileExpr, cond),
    ForExpr => helper!(ForExpr, range),
    IterateExpr => helper!(IterateExpr, cond),
    ElseClause => true,
    _ => false,
  }
}
