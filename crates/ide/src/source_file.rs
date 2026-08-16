use async_inc::InputId;
use line_index::{LineIndex, TextRange};
use parser::{LexedStr, parse};
use syntax::{Green, Indel, Red, ast::Root, build_syntax_tree, reparse};

pub type FileId = InputId<SourceFile>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
  text: String,
  syntax_tree: Green,
  index: LineIndex,
}

impl SourceFile {
  pub fn new(text: impl Into<String>) -> Self {
    let text = text.into();
    let lexed = LexedStr::new(&text);
    let output = parse(&lexed);
    let syntax_tree = build_syntax_tree(&lexed, &output);
    let index = LineIndex::new(&text);

    Self {
      text,
      syntax_tree,
      index,
    }
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  pub fn text_of(&self, range: TextRange) -> &str {
    &self.text[range]
  }

  pub fn green_tree(&self) -> &Green {
    &self.syntax_tree
  }

  pub fn red_tree(&self) -> Red {
    Red::new_root(self.syntax_tree.clone())
  }

  pub fn root(&self) -> Root {
    Root::new(self.red_tree()).unwrap()
  }

  pub fn index(&self) -> &LineIndex {
    &self.index
  }

  pub fn set_text(&mut self, text: impl Into<String>) {
    *self = Self::new(text.into());
  }

  pub fn apply_change(&self, indel: &Indel) -> Option<(Self, Red)> {
    let reparse = reparse(&self.red_tree(), &self.text, indel)?;
    let text = indel.apply_to(&self.text);
    let index = LineIndex::new(&text);

    Some((
      Self {
        text,
        syntax_tree: reparse.new,
        index,
      },
      reparse.old,
    ))
  }
}
