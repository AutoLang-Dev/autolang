use crate::{
  Reparser,
  SyntaxKind::*,
  T,
  infra::{lexed::LexedStr, parser::Parser},
};

macro_rules! parser {
  ($input:literal) => {{
    let input = $input;
    let parser_input = LexedStr::new(input).to_input();
    (input, Parser::new(parser_input))
  }};
}

macro_rules! output_snap {
  ($input:expr, $output:expr) => {{
    let output = $output;
    assert!(output.errors().is_empty());
    snap!($input, output.steps());
  }};
}

#[test]
fn detects_joint_composite_tokens() {
  let (input, mut parser) = parser!("a->b");

  parser.bump_any();
  assert!(parser.at(T![->]));
  parser.bump(T![->]);
  assert_eq!(parser.current(), Ident);

  output_snap!(input, parser.finish());
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

  output_snap!(input, parser.finish());
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

macro_rules! test_reparser {
  ($($input:ident => $output:ident),* $(,)?) => {
    $(
      assert_eq!(crate::reparser($input), Some(Reparser::$output));
    )*
  };
}

#[test]
fn reparser_maps_delimiter_scopes() {
  test_reparser! {
    ArgList => ArgList,
    ArrayExpr => ArrayExpr,
    RepeatExpr => ArrayExpr,
    ArrayType => ArrayOrSliceType,
    SliceType => ArrayOrSliceType,
    AttrInner => AttrInner,
    BlockExpr => BraceExpr,
    CaseArmList => CaseArmList,
    DelimitedTokenTree => DelimitedTokenTree,
    IndexArg => IndexArg,
    Module => Module,
    ParameterList => ParameterList,
    StructExpr => BraceExpr,
    StructType => StructType,
    ParenExpr => TupleOrParenExpr,
    TupleExpr => TupleOrParenExpr,
    ParenType => TupleOrParenType,
    TupleType => TupleOrParenType,
    UsingTreeList => UsingTreeList,
  }
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

macro_rules! reparser {
  ($input:literal, $old_kind:ident) => {{
    let input = $input;
    let parser_input = LexedStr::new(input).to_input();
    let output = crate::reparser($old_kind).unwrap().parse(parser_input);
    output_snap!(input, output);
  }};
}

#[test]
fn reparser_paren_to_tuple() {
  reparser!("(x, y)", ParenExpr);
}

#[test]
fn reparser_tuple_to_paren() {
  reparser!("(x)", TupleExpr);
}

#[test]
fn reparser_array_to_repeat() {
  reparser!("[1; 2]", ArrayExpr);
}

#[test]
fn reparser_repeat_to_array() {
  reparser!("[1, 2]", RepeatExpr);
}

#[test]
fn reparser_struct_to_block() {
  reparser!("{ x; }", StructExpr);
}

#[test]
fn reparser_block_to_struct() {
  reparser!("{ x }", BlockExpr);
}
