use crate::server::Server;
use line_index::{LineIndex, TextRange};
use lsp_types::Uri;
use parser::{LexedStr, parse};
use syntax::{Red, build_syntax_tree};

pub struct Document {
  text: String,
  index: LineIndex,
  syntax_tree: Red,
}

impl Document {
  pub fn new(text: String) -> Self {
    let index = LineIndex::new(&text);

    let lexed = LexedStr::new(&text);
    let output = parse(&lexed);
    let green = build_syntax_tree(&lexed, &output);
    let syntax_tree = Red::new_root(green);

    Self {
      text,
      index,
      syntax_tree,
    }
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  pub fn text_of(&self, range: TextRange) -> &str {
    &self.text[range]
  }

  pub fn index(&self) -> &LineIndex {
    &self.index
  }

  pub fn syntax_tree(&self) -> &Red {
    &self.syntax_tree
  }
}

impl Server {
  pub fn update_document(&mut self, uri: Uri, text: String) {
    self.documents.insert(uri, Document::new(text));
  }

  pub fn close_document(&mut self, uri: &Uri) {
    self.documents.remove(uri);
  }

  pub fn get_document(&self, uri: &Uri) -> Option<&Document> {
    self.documents.get(uri)
  }
}
