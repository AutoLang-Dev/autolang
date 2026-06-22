use crate::{
  SyntaxKind,
  infra::{
    event::{Error, Output, Step},
    lexed::LexedStr,
  },
};
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TreeStep<'text> {
  Enter(SyntaxKind),
  Exit,
  Token { kind: SyntaxKind, text: &'text str },
  Error(Error),
}

pub fn emit_tree_steps<'text>(
  lexed: &LexedStr<'text>,
  output: &Output,
  sink: &mut impl FnMut(TreeStep<'text>),
) -> bool {
  let mut raw_cursor = 0;
  let mut steps = output.steps();
  let mut has_root_exit = false;

  if let (Some(Step::Enter(first)), Some(Step::Exit)) = (steps.first(), steps.last()) {
    sink(TreeStep::Enter(*first));
    steps = &steps[1..steps.len() - 1];
    has_root_exit = true;
  }

  for step in steps {
    match *step {
      Step::Enter(kind) => {
        emit_trivia(lexed, &mut raw_cursor, sink);
        sink(TreeStep::Enter(kind));
      }
      Step::Exit => sink(TreeStep::Exit),
      Step::Token { kind, n_raw_tokens } => {
        emit_trivia(lexed, &mut raw_cursor, sink);
        emit_token(lexed, &mut raw_cursor, kind, n_raw_tokens, sink);
      }
      Step::Error(index) => {
        emit_trivia(lexed, &mut raw_cursor, sink);
        let error = output.errors()[index as usize];
        sink(TreeStep::Error(error));
      }
    }
  }

  emit_trivia(lexed, &mut raw_cursor, sink);

  if has_root_exit {
    sink(TreeStep::Exit);
  }

  raw_cursor == lexed.len()
}

fn emit_trivia<'text>(
  lexed: &LexedStr<'text>,
  raw_cursor: &mut u32,
  sink: &mut impl FnMut(TreeStep<'text>),
) {
  while *raw_cursor < lexed.len() {
    let kind = lexed.kind(*raw_cursor);
    if !kind.is_trivia() {
      break;
    }

    emit_token(lexed, raw_cursor, kind, 1, sink);
  }
}

fn emit_token<'text>(
  lexed: &LexedStr<'text>,
  raw_cursor: &mut u32,
  kind: SyntaxKind,
  n_raw_tokens: u8,
  sink: &mut impl FnMut(TreeStep<'text>),
) {
  let start = *raw_cursor;
  let end = start + u32::from(n_raw_tokens);
  assert!(end <= lexed.len());

  if !kind.is_trivia() {
    for raw in start..end {
      assert!(!lexed.kind(raw).is_trivia());
    }
  }

  let text = lexed.range_text(start..end);
  *raw_cursor = end;
  sink(TreeStep::Token { kind, text });
}

impl Debug for TreeStep<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TreeStep::Enter(kind) => write!(f, "Enter({kind:?})"),
      TreeStep::Exit => write!(f, "Exit"),
      TreeStep::Token { kind, text } => write!(f, "{kind:?}({text:?})"),
      TreeStep::Error(error) => write!(f, "Error({error:?})"),
    }
  }
}
