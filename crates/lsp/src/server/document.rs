use crate::server::semantic_tokens::{SemanticCache, SemanticDirty};
use line_index::{LineCol, LineIndex, TextRange, WideEncoding, WideLineCol};
use lsp_types::{Range, Uri};

use crate::server::{Document, ReparseTrace, Server};

impl Server {
  pub fn update_document(&mut self, uri: Uri, text: String) {
    let document = Document::new(text, SemanticDirty::Full);
    self.documents.insert(uri, document);
  }

  pub fn change_document_full(&mut self, uri: &Uri, text: String) -> Option<ReparseTrace> {
    if let Some(document) = self.documents.get_mut(uri) {
      *document = Document::new(text, SemanticDirty::Full);
      return Some(ReparseTrace::full(uri.clone()));
    }

    None
  }

  pub fn save_document(&mut self, uri: &Uri, text: Option<String>) -> Option<ReparseTrace> {
    if let Some(document) = self.documents.get_mut(uri) {
      document.force_full_reparse(text);
      return Some(ReparseTrace::full(uri.clone()));
    }

    None
  }

  pub fn change_document_range(
    &mut self,
    uri: &Uri,
    range: Range,
    text: String,
  ) -> Option<ReparseTrace> {
    if let Some(document) = self.documents.get_mut(uri) {
      return Some(document.apply_change(uri, range, text));
    }

    None
  }

  pub fn close_document(&mut self, uri: &Uri) {
    self.documents.remove(uri);
  }

  pub fn document(&self, uri: &Uri) -> Option<&Document> {
    self.documents.get(uri)
  }

  pub fn syntax_tree(&self, uri: &Uri) -> Option<String> {
    self
      .document(uri)
      .map(|document| syntax::debug::syntax_tree(&document.parse))
  }
}

impl Document {
  fn new(text: String, semantic_dirty: SemanticDirty) -> Self {
    let line_index = LineIndex::new(&text);
    let parse = syntax::parse(&text);

    Self {
      text,
      line_index,
      parse,
      semantic_cache: SemanticCache::default(),
      semantic_dirty,
    }
  }

  fn apply_change(&mut self, uri: &Uri, range: Range, insert: String) -> ReparseTrace {
    let Some(delete) = self.lsp_range_to_text_range(range) else {
      self.reparse_full(self.text.clone(), SemanticDirty::Full);
      return ReparseTrace::full(uri.clone());
    };

    let indel = syntax::Indel { delete, insert };
    let reparse = self.parse.reparse(&self.text, &indel);
    let edit_range = text_range_to_lsp(&self.line_index, delete);

    indel.apply(&mut self.text);
    let new_line_index = LineIndex::new(&self.text);
    let old_line_index = std::mem::replace(&mut self.line_index, new_line_index);

    match reparse {
      syntax::Reparse::Noop => ReparseTrace::noop(uri.clone(), edit_range),
      syntax::Reparse::Token(reparse) => {
        let dirty = lsp_dirty_range(&old_line_index, &self.line_index, reparse.dirty);
        self.parse = reparse.parse;
        self.mark_semantic_dirty(SemanticDirty::Precise {
          old_range: reparse.dirty.old,
          new_range: reparse.dirty.new,
        });
        ReparseTrace::token(uri.clone(), edit_range, dirty)
      }
      syntax::Reparse::Node(reparse) => {
        let dirty = lsp_dirty_range(&old_line_index, &self.line_index, reparse.dirty);
        let reparser = format!("{:?}", reparse.reparser);
        let old_kind = reparse.old_kind;
        let new_kind = reparse.new_kind;
        self.parse = reparse.parse;
        self.mark_semantic_dirty(SemanticDirty::Precise {
          old_range: reparse.dirty.old,
          new_range: reparse.dirty.new,
        });
        ReparseTrace::node(uri.clone(), edit_range, dirty, reparser, old_kind, new_kind)
      }
      syntax::Reparse::Full(parse) => {
        self.parse = parse;
        self.mark_semantic_dirty(SemanticDirty::Full);
        ReparseTrace::full(uri.clone())
      }
    }
  }

  fn reparse_full(&mut self, text: String, semantic_dirty: SemanticDirty) {
    self.text = text;
    self.line_index = LineIndex::new(&self.text);
    self.parse = syntax::parse(&self.text);
    self.mark_semantic_dirty(semantic_dirty);
  }

  fn force_full_reparse(&mut self, text: Option<String>) {
    if let Some(text) = text {
      self.text = text;
      self.line_index = LineIndex::new(&self.text);
    }

    self.parse = syntax::parse(&self.text);
    self.mark_semantic_dirty(SemanticDirty::Full);
  }

  fn mark_semantic_dirty(&mut self, dirty: SemanticDirty) {
    self.semantic_dirty = match (&self.semantic_dirty, dirty) {
      (SemanticDirty::Clean, dirty) => dirty,
      (SemanticDirty::Precise { .. }, SemanticDirty::Precise { .. }) => SemanticDirty::Full,
      (_, SemanticDirty::Full) | (SemanticDirty::Full, _) => SemanticDirty::Full,
      (_, SemanticDirty::Clean) => self.semantic_dirty.clone(),
    };
  }

  fn lsp_range_to_text_range(&self, range: Range) -> Option<TextRange> {
    let start = self.lsp_position_to_offset(range.start)?;
    let end = self.lsp_position_to_offset(range.end)?;
    Some(TextRange::new(start, end))
  }

  fn lsp_position_to_offset(&self, position: lsp_types::Position) -> Option<syntax::TextSize> {
    let wide = WideLineCol {
      line: position.line,
      col: position.character,
    };
    let line_col: LineCol = self.line_index.to_utf8(WideEncoding::Utf16, wide)?;
    self.line_index.offset(line_col)
  }
}

fn lsp_dirty_range(
  old_line_index: &LineIndex,
  new_line_index: &LineIndex,
  dirty: syntax::DirtyRange,
) -> crate::server::reparse_trace::TraceDirtyRange {
  crate::server::reparse_trace::TraceDirtyRange {
    old: text_range_to_lsp(old_line_index, dirty.old),
    new: text_range_to_lsp(new_line_index, dirty.new),
  }
}

fn text_range_to_lsp(line_index: &LineIndex, range: TextRange) -> Range {
  Range {
    start: position(line_index, range.start()),
    end: position(line_index, range.end()),
  }
}

fn position(line_index: &LineIndex, offset: syntax::TextSize) -> lsp_types::Position {
  let line_col = line_index.line_col(offset);
  let line_col = line_index
    .to_wide(WideEncoding::Utf16, line_col)
    .expect("valid UTF-16 position");
  lsp_types::Position::new(line_col.line, line_col.col)
}
