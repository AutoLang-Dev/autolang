use parser::{LexedStr, parse};
use syntax::{Green, Indel, Red, build_syntax_tree, reparse};
use text_size::TextRange;

pub struct SourceFile {
  text: String,
  syntax_tree: Green,
}

impl SourceFile {
  pub fn new(text: impl Into<String>) -> Self {
    let text = text.into();
    let lexed = LexedStr::new(&text);
    let output = parse(&lexed);
    let syntax_tree = build_syntax_tree(&lexed, &output);

    Self { text, syntax_tree }
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

  pub fn set_text(&mut self, text: impl Into<String>) {
    *self = Self::new(text.into());
  }

  pub fn apply_change(&self, indel: &Indel) -> Option<(Self, Red)> {
    let reparse = reparse(&self.red_tree(), &self.text, indel)?;

    Some((
      Self {
        text: indel.apply_to(&self.text),
        syntax_tree: reparse.new,
      },
      reparse.old,
    ))
  }
}
