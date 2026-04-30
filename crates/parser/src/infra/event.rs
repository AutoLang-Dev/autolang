use crate::SyntaxKind;
use std::{fmt::Debug, mem, num::NonZeroU32};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
  steps: Vec<Step>,
  errors: Vec<Error>,
}

impl Output {
  pub(crate) fn new(steps: Vec<Step>, errors: Vec<Error>) -> Self {
    Self { steps, errors }
  }

  pub fn steps(&self) -> &[Step] {
    &self.steps
  }

  pub fn errors(&self) -> &[Error] {
    &self.errors
  }

  pub fn into_parts(self) -> (Vec<Step>, Vec<Error>) {
    (self.steps, self.errors)
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Step {
  Enter(SyntaxKind),
  Exit,
  Token { kind: SyntaxKind, n_raw_tokens: u8 },
  Error(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
  Expected {
    expected: SyntaxKind,
    actual: SyntaxKind,
  },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
  Start {
    kind: SyntaxKind,
    forward_parent: Option<NonZeroU32>,
  },
  Finish,
  Token {
    kind: SyntaxKind,
    n_raw_tokens: u8,
  },
  Error(u32),
  Placeholder,
}

pub fn process(mut events: Vec<Event>, errors: Vec<Error>) -> Output {
  let mut steps = Vec::new();

  for i in 0..events.len() {
    match mem::replace(&mut events[i], Event::Placeholder) {
      Event::Start {
        kind,
        forward_parent,
      } => {
        let mut kinds = vec![kind];
        let mut forward_parent = forward_parent;

        let mut parent_idx = i;
        while let Some(distance) = forward_parent {
          parent_idx += distance.get() as usize;
          match mem::replace(&mut events[parent_idx], Event::Placeholder) {
            Event::Start {
              kind,
              forward_parent: next_forward_parent,
            } => {
              kinds.push(kind);
              forward_parent = next_forward_parent;
            }
            event => panic!("expected forwarded start event, got {event:?}"),
          }
        }

        for kind in kinds.into_iter().rev() {
          steps.push(Step::Enter(kind));
        }
      }
      Event::Finish => steps.push(Step::Exit),
      Event::Token { kind, n_raw_tokens } => steps.push(Step::Token { kind, n_raw_tokens }),
      Event::Error(index) => steps.push(Step::Error(index)),
      Event::Placeholder => (),
    }
  }

  Output::new(steps, errors)
}

impl Debug for Step {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Step::Enter(kind) => write!(f, "Enter({kind:?})"),
      Step::Exit => write!(f, "Exit"),
      Step::Token { kind, n_raw_tokens } => write!(f, "{kind:?}({n_raw_tokens})"),
      Step::Error(index) => write!(f, "Error({index})"),
    }
  }
}
