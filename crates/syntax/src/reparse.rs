mod errors;
mod node;
mod range;
mod token;

use crate::{Parse, SyntaxKind, TextRange, TextSize};
use range::{delimiters_balanced, range_text};

impl Parse {
  pub fn reparse(&self, old_text: &str, indel: &Indel) -> Reparse {
    if range_text(old_text, indel.delete) == indel.insert {
      return Reparse::Noop;
    }

    let new_text = indel.apply_to(old_text);
    if !check_reparse_preconditions(old_text, &new_text, indel) {
      return Reparse::Full(crate::parse(&new_text));
    }

    if let Some(reparse) = token::reparse_token(self, old_text, indel) {
      return Reparse::Token(reparse);
    }

    if !check_balanced_delimiter_diff(old_text, indel) {
      return Reparse::Full(crate::parse(&new_text));
    }

    if let Some(reparse) = node::reparse_node(self, old_text, &new_text, indel) {
      return Reparse::Node(reparse);
    }

    Reparse::Full(crate::parse(&new_text))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indel {
  pub delete: TextRange,
  pub insert: String,
}

impl Indel {
  pub fn apply(&self, text: &mut String) {
    let start = usize::from(self.delete.start());
    let end = usize::from(self.delete.end());
    text.replace_range(start..end, &self.insert);
  }

  fn apply_to(&self, text: &str) -> String {
    let mut text = text.to_string();
    self.apply(&mut text);
    text
  }

  fn insert_len(&self) -> TextSize {
    TextSize::of(&self.insert)
  }

  fn delta(&self) -> i64 {
    i64::from(u32::from(self.insert_len())) - i64::from(u32::from(self.delete.len()))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reparse {
  Noop,
  Token(TokenReparse),
  Node(NodeReparse),
  Full(Parse),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenReparse {
  pub parse: Parse,
  pub dirty: DirtyRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeReparse {
  pub parse: Parse,
  pub dirty: DirtyRange,
  pub reparser: parser::Reparser,
  pub old_kind: SyntaxKind,
  pub new_kind: SyntaxKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRange {
  pub old: TextRange,
  pub new: TextRange,
}

fn check_reparse_preconditions(old_text: &str, new_text: &str, indel: &Indel) -> bool {
  !(indel.delete.start() == TextSize::from(0)
    && (old_text.starts_with("#!") || new_text.starts_with("#!")))
}

fn check_balanced_delimiter_diff(old_text: &str, indel: &Indel) -> bool {
  let old_delete = range_text(old_text, indel.delete);
  delimiters_balanced(old_delete) && delimiters_balanced(&indel.insert)
}
