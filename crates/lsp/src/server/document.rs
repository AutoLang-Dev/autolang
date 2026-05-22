use crate::server::Server;
use line_index::{LineIndex, TextRange, TextSize, WideEncoding, WideLineCol};
use lsp_types::{Position, Range, Uri};
use parser::{LexedStr, parse};
use syntax::{Indel, Red, build_syntax_tree, reparse};

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

  pub fn set_text(&mut self, text: impl Into<String>) {
    *self = Self::new(text.into());
  }

  pub fn apply_change(&mut self, indel: &Indel) -> Option<TextRange> {
    let reparse = reparse(&self.syntax_tree, &self.text, indel)?;

    let text = indel.apply_to(&self.text);
    let index = LineIndex::new(&text);
    let syntax_tree = Red::new_root(reparse.new);

    *self = Self {
      text,
      index,
      syntax_tree,
    };

    Some(reparse.old.range())
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
