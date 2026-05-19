use parser::SyntaxKind;
use rgt::{green, lang::Language, red};

#[derive(Debug, Clone)]
pub enum DiagPayload {
  Subtree,
  Diag(parser::Error),
}

#[derive(Debug, Clone, Default)]
pub struct Payload {
  pub diag: Option<DiagPayload>,
}

pub enum AutoLang {}

impl Language for AutoLang {
  type Kind = SyntaxKind;
  type Payload = Payload;

  fn compose_node(_: SyntaxKind, children: &[Green]) -> Payload {
    let diag = children
      .iter()
      .any(|child| child.payload().diag.is_some())
      .then_some(DiagPayload::Subtree);

    Payload { diag }
  }
}

pub type Green = green::Green<AutoLang>;
pub type Red = red::Red<AutoLang>;
