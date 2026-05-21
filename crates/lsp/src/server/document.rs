use crate::server::Server;
use line_index::{LineIndex, TextRange};
use lsp_types::Uri;

pub struct Document {
  text: String,
  index: LineIndex,
}

impl Document {
  pub fn new(text: String) -> Self {
    let index = LineIndex::new(&text);
    Self { text, index }
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
