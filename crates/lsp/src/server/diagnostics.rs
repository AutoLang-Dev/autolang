use crate::server::{Document, Server};
use line_index::{TextRange, TextSize, WideEncoding};
use locale::tr;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, PublishDiagnosticsParams, Range, Uri};

impl Server {
  pub fn diagnostics(&self, uri: &Uri) -> PublishDiagnosticsParams {
    let diagnostics = self
      .document(uri)
      .map(|document| document.diagnostics())
      .unwrap_or_default();

    PublishDiagnosticsParams::new(uri.clone(), diagnostics, None)
  }

  pub fn empty_diagnostics(uri: Uri) -> PublishDiagnosticsParams {
    PublishDiagnosticsParams::new(uri, Vec::new(), None)
  }
}

impl Document {
  fn diagnostics(&self) -> Vec<Diagnostic> {
    let root = self.parse.syntax_node();

    self
      .parse
      .errors()
      .iter()
      .map(|error| {
        let range = self.error_range(&root, error.offset);
        let mut diagnostic = Diagnostic::new_simple(range, error_message(error.error));
        diagnostic.severity = Some(DiagnosticSeverity::ERROR);
        diagnostic.source = Some("autolang".to_string());
        diagnostic
      })
      .collect()
  }

  fn error_range(&self, root: &syntax::SyntaxNode, offset: u32) -> Range {
    let offset = TextSize::new(offset.min(self.text.len() as u32));
    let range = root
      .descendants_with_tokens()
      .filter_map(|element| element.into_token())
      .find_map(|token| {
        token
          .text_range()
          .contains(offset)
          .then(|| token.text_range())
      })
      .unwrap_or_else(|| TextRange::empty(offset));

    self.text_range_to_lsp(range)
  }

  pub fn text_range_to_lsp(&self, range: TextRange) -> Range {
    Range {
      start: self.position(range.start()),
      end: self.position(range.end()),
    }
  }

  pub fn position(&self, offset: TextSize) -> Position {
    let line_col = self.line_index.line_col(offset);
    let line_col = self
      .line_index
      .to_wide(WideEncoding::Utf16, line_col)
      .expect("valid UTF-16 position");
    Position::new(line_col.line, line_col.col)
  }
}

fn error_message(error: syntax::Error) -> String {
  match error {
    syntax::Error::Expected { expected, actual } => tr().diagnostic_expected_got(expected, actual),
  }
}
