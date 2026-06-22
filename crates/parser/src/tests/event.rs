use crate::{
  SyntaxKind::*,
  infra::event::{Event, process},
};
use std::num::NonZeroU32;

#[test]
fn process_forward_parent() {
  let output = process(
    vec![
      Event::Start {
        kind: PathExpr,
        forward_parent: Some(NonZeroU32::new(3).unwrap()),
      },
      Event::Token {
        kind: Ident,
        n_raw_tokens: 1,
      },
      Event::Finish,
      Event::Start {
        kind: CallExpr,
        forward_parent: None,
      },
      Event::Token {
        kind: OpenParen,
        n_raw_tokens: 1,
      },
      Event::Token {
        kind: CloseParen,
        n_raw_tokens: 1,
      },
      Event::Finish,
    ],
    Vec::new(),
  );

  snap!("forward parent", output.steps());
}

#[test]
fn process_forward_parent_chain() {
  let output = process(
    vec![
      Event::Start {
        kind: PathExpr,
        forward_parent: Some(NonZeroU32::new(3).unwrap()),
      },
      Event::Token {
        kind: Ident,
        n_raw_tokens: 1,
      },
      Event::Finish,
      Event::Start {
        kind: FieldExpr,
        forward_parent: Some(NonZeroU32::new(4).unwrap()),
      },
      Event::Token {
        kind: Dot,
        n_raw_tokens: 1,
      },
      Event::Token {
        kind: Ident,
        n_raw_tokens: 1,
      },
      Event::Finish,
      Event::Start {
        kind: CallExpr,
        forward_parent: None,
      },
      Event::Token {
        kind: OpenParen,
        n_raw_tokens: 1,
      },
      Event::Token {
        kind: CloseParen,
        n_raw_tokens: 1,
      },
      Event::Finish,
    ],
    Vec::new(),
  );

  snap!("forward parent chain", output.steps());
}
