use crate::server::{Server, document::Document};
use ide::{Diagnostic, collect_diag};
use locale::tr;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
  Diagnostic as LspDiagnostic, DiagnosticSeverity, PublishDiagnosticsParams, Uri,
  notification::{Notification as _, PublishDiagnostics},
};
use parser::Error;

impl Document {
  pub fn diagnostics(&self) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    collect_diag(&mut diags, &self.file.red_tree());
    diags
  }
}

fn error_message(err: Error) -> String {
  use Error::*;
  match err {
    Expected { expected, actual } => tr().diagnostic_expected_got(expected, actual),
  }
}

impl Server {
  pub fn publish_diagnostic(&self, uri: &Uri, conn: &Connection) -> anyhow::Result<()> {
    let diags = self
      .get_document(uri)
      .map(|doc| {
        doc
          .diagnostics()
          .iter()
          .map(|diag| {
            let range = doc.span_to_lsp_range(diag.range);
            let mut diag = LspDiagnostic::new_simple(range, error_message(diag.error));
            diag.severity = Some(DiagnosticSeverity::ERROR);
            diag.source = Some("autolang".into());
            diag
          })
          .collect()
      })
      .unwrap_or_default();

    let params = PublishDiagnosticsParams::new(uri.clone(), diags, None);
    conn.sender.send(Message::Notification(Notification::new(
      PublishDiagnostics::METHOD.to_string(),
      params,
    )))?;

    Ok(())
  }
}
