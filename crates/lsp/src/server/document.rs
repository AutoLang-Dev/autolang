use crate::server::Server;
use ide::SourceFile;
use line_index::{LineIndex, TextRange, TextSize, WideEncoding, WideLineCol};
use lsp_types::{Position, Range, Uri};
use syntax::Indel;

pub struct Document {
  pub file: SourceFile,
  pub index: LineIndex,
}

impl Document {
  pub fn new(text: impl Into<String>) -> Self {
    let file = SourceFile::new(text);
    let index = LineIndex::new(file.text());
    Self { index, file }
  }

  pub fn set_text(&mut self, text: impl Into<String>) {
    self.file.set_text(text);
    self.index = LineIndex::new(self.file.text());
  }

  pub fn apply_change(&mut self, indel: &Indel) -> Option<TextRange> {
    let (file, old) = self.file.apply_change(indel)?;
    self.file = file;
    self.index = LineIndex::new(self.file.text());
    Some(old.range())
  }

  pub fn lsp_range_to_span(&self, range: Range) -> TextRange {
    let start = self.lsp_position_to_offset(range.start);
    let end = self.lsp_position_to_offset(range.end);
    TextRange::new(start, end)
  }

  pub fn span_to_lsp_range(&self, range: TextRange) -> Range {
    let start = self.offset_to_lsp_position(range.start());
    let end = self.offset_to_lsp_position(range.end());
    Range::new(start, end)
  }

  pub fn lsp_position_to_offset(&self, pos: Position) -> TextSize {
    let line_col = WideLineCol {
      line: pos.line,
      col: pos.character,
    };
    let line_col = self.index.to_utf8(WideEncoding::Utf16, line_col).unwrap();
    self.index.offset(line_col).unwrap()
  }

  pub fn offset_to_lsp_position(&self, offset: TextSize) -> Position {
    let line_col = self.index.line_col(offset);
    let line_col = self.index.to_wide(WideEncoding::Utf16, line_col).unwrap();
    Position::new(line_col.line, line_col.col)
  }
}

impl Server {
  pub fn update_document(&mut self, uri: &Uri, text: String) {
    self.documents.insert(uri.clone(), Document::new(text));
  }

  pub fn close_document(&mut self, uri: &Uri) {
    self.documents.remove(uri);
  }

  pub fn get_document(&self, uri: &Uri) -> Option<&Document> {
    self.documents.get(uri)
  }

  pub fn get_document_mut(&mut self, uri: &Uri) -> Option<&mut Document> {
    self.documents.get_mut(uri)
  }
}
