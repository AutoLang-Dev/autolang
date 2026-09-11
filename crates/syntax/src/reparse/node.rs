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
  TuplePat,
  RecordPat,
  DelimitedTokenTree,
  IndexArg,
  Module,
  TupleType(bool),
  RecordType(bool),
  TupleOrParenExpr,
  UsingTreeList,
}

impl Reparser {
  pub fn new(node: &Red, delete: TextRange) -> Option<Self> {
    let is_nominal = || {
      let first = node.prev_token().unwrap().prev_token().unwrap();
      if first.kind() == KwNominal {
        return true;
      }
      let Some(first) = first.prev_token() else {
        return false;
      };
      if first.kind() == KwNominal {
        return true;
      }
      let Some(first) = first.prev_token() else {
        return false;
      };
      first.kind() == KwNominal
    };

    let reparser = match node.kind() {
      ArgList => Self::ArgList,
      ArrayExpr | RepeatExpr => Self::ArrayExpr,
      ArrayType | SliceType => Self::ArrayOrSliceType,
      AttrInner => Self::AttrInner,
      BlockExpr if must_be_block(node) => Self::BlockExpr,
      BlockExpr | RecordExpr => Self::BraceExpr,
      DelimitedTokenTree => Self::DelimitedTokenTree,
      IndexArg => Self::IndexArg,
      Module => Self::Module,
      RecordType => Self::RecordType(is_nominal()),
      TupleType => Self::TupleType(is_nominal()),
      TuplePat => Self::TuplePat,
      RecordPat => Self::RecordPat,
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

    match self {
      Self::ArgList => expr::arg_list(&mut parser),
      Self::ArrayExpr => expr::array_expr(&mut parser),
      Self::ArrayOrSliceType => types::array_or_slice_type(&mut parser),
      Self::AttrInner => attrs::attr_inner(&mut parser),
      Self::BlockExpr => expr::block_expr(&mut parser),
      Self::BraceExpr => expr::brace_expr(&mut parser),
      Self::DelimitedTokenTree => token_trees::delimited_token_tree(&mut parser),
      Self::IndexArg => expr::index_arg(&mut parser),
      Self::Module => items::module(&mut parser),
      Self::TupleType(nominal) => types::tuple_type(&mut parser, false, nominal),
      Self::RecordType(nominal) => types::record_type(&mut parser, false, nominal),
      Self::TupleOrParenExpr => expr::paren_expr(&mut parser, false),
      Self::UsingTreeList => items::using_tree_list(&mut parser),
      Self::TuplePat => pat::tuple_pat(&mut parser),
      Self::RecordPat => pat::record_pat(&mut parser),
    };

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
    IterateExpr => helper!(IterateExpr, init),
    ElseClause => true,
    _ => false,
  }
}
