use crate::grammar::items::module;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reparser {
  ArgList,
  ArrayExpr,
  ArrayOrSliceType,
  AttrInner,
  BraceExpr,
  CaseArmList,
  DelimitedTokenTree,
  IndexArg,
  Module,
  ParameterList,
  StructType,
  TupleOrParenExpr,
  TupleOrParenType,
  UsingTreeList,
}

impl Reparser {
  pub fn parse(self, input: Input) -> Output {
    let mut parser = Parser::new(input);

    match self {
      Reparser::ArgList => expr::arg_list(&mut parser),
      Reparser::ArrayExpr => expr::array_expr(&mut parser),
      Reparser::ArrayOrSliceType => types::array_or_slice_type(&mut parser),
      Reparser::AttrInner => attrs::attr_inner(&mut parser),
      Reparser::BraceExpr => expr::brace_expr(&mut parser),
      Reparser::CaseArmList => expr::case_arm_list(&mut parser),
      Reparser::DelimitedTokenTree => token_trees::delimited_token_tree(&mut parser),
      Reparser::IndexArg => expr::index_arg(&mut parser),
      Reparser::Module => module(&mut parser),
      Reparser::ParameterList => func::parameter_list(&mut parser),
      Reparser::StructType => types::struct_type(&mut parser),
      Reparser::TupleOrParenExpr => expr::tuple_or_paren_expr(&mut parser),
      Reparser::TupleOrParenType => types::tuple_or_paren_type(&mut parser),
      Reparser::UsingTreeList => items::using_tree_list(&mut parser),
    };

    parser.finish()
  }
}

pub fn reparser(kind: SyntaxKind) -> Option<Reparser> {
  match kind {
    SyntaxKind::ArgList => Some(Reparser::ArgList),
    SyntaxKind::ArrayExpr | SyntaxKind::RepeatExpr => Some(Reparser::ArrayExpr),
    SyntaxKind::ArrayType | SyntaxKind::SliceType => Some(Reparser::ArrayOrSliceType),
    SyntaxKind::AttrInner => Some(Reparser::AttrInner),
    SyntaxKind::BlockExpr | SyntaxKind::StructExpr => Some(Reparser::BraceExpr),
    SyntaxKind::CaseArmList => Some(Reparser::CaseArmList),
    SyntaxKind::DelimitedTokenTree => Some(Reparser::DelimitedTokenTree),
    SyntaxKind::IndexArg => Some(Reparser::IndexArg),
    SyntaxKind::Module => Some(Reparser::Module),
    SyntaxKind::ParameterList => Some(Reparser::ParameterList),
    SyntaxKind::StructType => Some(Reparser::StructType),
    SyntaxKind::ParenExpr | SyntaxKind::TupleExpr => Some(Reparser::TupleOrParenExpr),
    SyntaxKind::ParenType | SyntaxKind::TupleType => Some(Reparser::TupleOrParenType),
    SyntaxKind::UsingTreeList => Some(Reparser::UsingTreeList),
    _ => None,
  }
}
