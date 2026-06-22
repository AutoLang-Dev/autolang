use crate::{
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
