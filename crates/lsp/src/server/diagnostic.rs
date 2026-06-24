use crate::server::{Server, document::Document};
use locale::tr;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
  Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams, Uri,
  notification::{Notification as _, PublishDiagnostics},
};
use parser::Error;
use syntax::{DiagPayload, Red};

impl Document {
  pub fn diagnostics(&self) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    collect_diag(&mut diags, self.syntax_tree(), self);
    diags
  }
}

fn collect_diag(diags: &mut Vec<Diagnostic>, tree: &Red, doc: &Document) {
  let Some(diag) = tree.payload().diag.as_ref() else {
    return;
  };

  match diag {
    DiagPayload::Diag(err) => {
      let range = doc.span_to_lsp_range(tree.range());

      let mut diag = Diagnostic::new_simple(range, error_message(*err));
      diag.severity = Some(DiagnosticSeverity::ERROR);
      diag.source = Some("autolang".into());

      diags.push(diag);
    }
    _ => {
      for child in tree.children() {
        collect_diag(diags, &child, doc);
      }
    }
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
      .map(|doc| doc.diagnostics())
      .unwrap_or_default();

    let params = PublishDiagnosticsParams::new(uri.clone(), diags, None);
    conn.sender.send(Message::Notification(Notification::new(
      PublishDiagnostics::METHOD.to_string(),
      params,
    )))?;

    Ok(())
  }
}
