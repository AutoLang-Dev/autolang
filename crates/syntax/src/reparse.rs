mod node;
mod token;

use crate::{Green, Red, build_syntax_tree};
use parser::{LexedStr, parse};
use text_size::{TextRange, TextSize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indel {
  pub delete: TextRange,
  pub insert: String,
}

impl Indel {
  pub fn deleted<'a>(&self, text: &'a str) -> &'a str {
    &text[self.delete]
  }

  pub fn apply(&self, text: &mut String) {
    let start: usize = self.delete.start().into();
    let end: usize = self.delete.end().into();
    text.replace_range(start..end, &self.insert);
  }

  pub fn apply_to(&self, text: &str) -> String {
    let mut text = text.to_string();
    self.apply(&mut text);
    text
  }

  pub fn insert_len(&self) -> TextSize {
    TextSize::of(&self.insert)
  }
}

#[derive(Clone)]
pub struct Reparse {
  pub old: Red,
  pub new: Green,
}

pub fn reparse(tree: &Red, source: &str, indel: &Indel) -> Option<Reparse> {
  if indel.deleted(source) == indel.insert {
    return None;
  }

  if can_local_reparse(source, indel)
    && let Some(mut node) = tree.covering_node(indel.delete)
  {
    let mut replacement = None;

    if replacement.is_none() {
      replacement = token::reparse(&node, source, indel);
    }
    if replacement.is_none() {
      replacement = node::reparse(&mut node, source, indel);
    }

    if let Some(replacement) = replacement {
      return Some(Reparse {
        old: node.clone(),
        new: node.replace_with(replacement),
      });
    }
  }

  let source = indel.apply_to(source);
  let lexed = LexedStr::new(&source);
  let output = parse(&lexed);
  Some(Reparse {
    old: tree.clone(),
    new: build_syntax_tree(&lexed, &output),
  })
}

fn can_local_reparse(source: &str, indel: &Indel) -> bool {
  for check in CHECKERS {
    if check(indel.deleted(source)) {
      return false;
    }
    if check(&indel.insert) {
      return false;
    }
  }
  true
}

const CHECKERS: [fn(&str) -> bool; 3] = [has_delimiter, has_qoute, has_comment];

// Accepts false positives (e.g., brackets inside strings/comments).
fn has_delimiter(text: &str) -> bool {
  text.contains(['(', ')', '[', ']', '{', '}'])
}

fn has_qoute(text: &str) -> bool {
  text.contains(['"', '\''])
}

fn has_comment(text: &str) -> bool {
  text.find("//").is_some()
}
