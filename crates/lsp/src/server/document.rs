use line_index::LineIndex;
use lsp_types::Uri;

use crate::server::Server;

impl Server {
  pub fn update_document(&mut self, uri: Uri, text: String) {
    let index = LineIndex::new(&text);
    self.documents.insert(uri, (text, index));
  }

  pub fn close_document(&mut self, uri: &Uri) {
    self.documents.remove(uri);
  }

  pub fn get_document(&self, uri: &Uri) -> Option<(&str, &LineIndex)> {
    self.documents.get(uri).map(|(s, i)| (s.as_str(), i))
  }
}
