use crate::{
  Reparser,
  SyntaxKind::*,
  T,
  infra::{lexed::LexedStr, parser::Parser},
};

macro_rules! parser {
  ($input:literal) => {{
    let input = $input;
    let parser_input = LexedStr::lex(input).to_input();
    (input, Parser::new(parser_input))
  }};
}

macro_rules! finish_snap {
  ($input:expr, $parser:expr) => {{
    let (steps, errors) = $parser.finish().into_parts();
    assert!(errors.is_empty());
    snap!($input, steps);
  }};
}

#[test]
fn detects_joint_composite_tokens() {
  let (input, mut parser) = parser!("a->b");

  parser.bump_any();
  assert!(parser.at(T![->]));
  parser.bump(T![->]);
  assert_eq!(parser.current(), Ident);

  finish_snap!(input, parser);
}

#[test]
fn does_not_detect_composite_tokens_across_trivia() {
  let (_, mut parser) = parser!("a- >b");

  parser.bump_any();
  assert!(!parser.at(T![->]));
  assert_eq!(parser.current(), Minus);
}

#[test]
fn bump_any_consumes_only_one_raw_token() {
  let (input, mut parser) = parser!("a->b");

  parser.bump_any();
  parser.bump_any();
  assert_eq!(parser.current(), Gt);

  finish_snap!(input, parser);
}

#[test]
fn marker_complete_defuses_bomb() {
  let (_, mut parser) = parser!("");
  let marker = parser.start();

  parser.complete(marker, SourceFile);
}

#[test]
fn marker_abandon_defuses_bomb() {
  let (_, mut parser) = parser!("");
  let marker = parser.start();

  parser.abandon(marker);

  let (steps, errors) = parser.finish().into_parts();
  assert!(steps.is_empty());
  assert!(errors.is_empty());
}

#[test]
#[should_panic(expected = "marker must be completed or abandoned")]
fn marker_drop_without_complete_or_abandon_panics() {
  let (_, mut parser) = parser!("");

  let _marker = parser.start();
}

#[test]
fn reparser_maps_delimiter_scopes() {
  assert_eq!(crate::reparser(ArgList), Some(Reparser::ArgList));
  assert_eq!(crate::reparser(ArrayExpr), Some(Reparser::ArrayExpr));
  assert_eq!(crate::reparser(RepeatExpr), Some(Reparser::ArrayExpr));
  assert_eq!(crate::reparser(ArrayType), Some(Reparser::ArrayOrSliceType));
  assert_eq!(crate::reparser(SliceType), Some(Reparser::ArrayOrSliceType));
  assert_eq!(crate::reparser(AttrInner), Some(Reparser::AttrInner));
  assert_eq!(crate::reparser(BlockExpr), Some(Reparser::BraceExpr));
  assert_eq!(crate::reparser(CaseArmList), Some(Reparser::CaseArmList));
  assert_eq!(
    crate::reparser(DelimitedTokenTree),
    Some(Reparser::DelimitedTokenTree)
  );
  assert_eq!(crate::reparser(IndexArg), Some(Reparser::IndexArg));
  assert_eq!(crate::reparser(ModuleInner), Some(Reparser::ModuleInner));
  assert_eq!(
    crate::reparser(ParameterList),
    Some(Reparser::ParameterList)
  );
  assert_eq!(crate::reparser(StructExpr), Some(Reparser::BraceExpr));
  assert_eq!(crate::reparser(StructType), Some(Reparser::StructType));
  assert_eq!(crate::reparser(ParenExpr), Some(Reparser::TupleOrParenExpr));
  assert_eq!(crate::reparser(TupleExpr), Some(Reparser::TupleOrParenExpr));
  assert_eq!(crate::reparser(ParenType), Some(Reparser::TupleOrParenType));
  assert_eq!(crate::reparser(TupleType), Some(Reparser::TupleOrParenType));
  assert_eq!(
    crate::reparser(UsingTreeList),
    Some(Reparser::UsingTreeList)
  );
}

#[test]
fn reparser_does_not_map_items() {
  for kind in [
    BindingItem,
    FunctionItem,
    TypeItem,
    ImplItem,
    AssociatedItem,
    UsingItem,
    ModuleItem,
    EmptyItem,
    SourceFile,
  ] {
    assert_eq!(crate::reparser(kind), None);
  }
}

#[test]
fn grouped_reparser_allows_expr_kind_mutation() {
  let lexed = LexedStr::relex("(x, y)");
  let output = Reparser::TupleOrParenExpr.parse(&lexed);
  let (steps, errors) = output.into_parts();

  assert!(errors.is_empty());
  assert!(
    steps
      .iter()
      .any(|step| matches!(step, crate::Step::Enter(TupleExpr)))
  );
}

#[test]
fn grouped_reparser_allows_brace_expr_kind_mutation() {
  let lexed = LexedStr::relex("{ x: 1 }");
  let output = Reparser::BraceExpr.parse(&lexed);
  let (steps, errors) = output.into_parts();

  assert!(errors.is_empty());
  assert!(
    steps
      .iter()
      .any(|step| matches!(step, crate::Step::Enter(StructExpr)))
  );
}

#[test]
fn brace_reparser_parses_single_shorthand_as_struct_expr() {
  let lexed = LexedStr::relex("{ x }");
  let output = Reparser::BraceExpr.parse(&lexed);
  let (steps, errors) = output.into_parts();

  assert!(errors.is_empty());
  assert!(
    steps
      .iter()
      .any(|step| matches!(step, crate::Step::Enter(StructExpr)))
  );
}
