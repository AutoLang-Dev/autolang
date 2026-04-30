use crate::{
  SyntaxKind,
  infra::{input::Input, lexed::LexedStr},
};
use std::fmt::Debug;

#[derive(PartialEq, Eq)]
struct InputToken {
  kind: SyntaxKind,
  joint: bool,
}

impl Debug for InputToken {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.joint {
      write!(f, "{:?} joint", self.kind)
    } else {
      write!(f, "{:?}", self.kind)
    }
  }
}

fn input_tokens(input: &Input) -> Vec<InputToken> {
  (0..input.len())
    .map(|idx| InputToken {
      kind: input.kind(idx),
      joint: input.is_joint(idx),
    })
    .collect()
}

macro_rules! input_snap {
  ($input:literal) => {{
    let input = $input;
    let lexed = LexedStr::new(input);
    let parser_input = lexed.to_input();
    snap!(input, input_tokens(&parser_input));
    parser_input
  }};
}

#[test]
fn marks_adjacent_compound_tokens_joint() {
  let parser_input = input_snap!("a->b");

  assert!(parser_input.is_joint(1));
}

#[test]
fn does_not_mark_compound_tokens_joint_across_trivia() {
  let parser_input = input_snap!("a- >b");

  assert!(!parser_input.is_joint(1));
}
