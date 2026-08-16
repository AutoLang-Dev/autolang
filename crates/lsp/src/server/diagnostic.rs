use crate::server::span_to_lsp_range;
use async_inc::{Query, QueryCtx};
use async_lsp::lsp_types::{Diagnostic as LspDiagnostic, DiagnosticSeverity};
use ide::{FileId, GetDiagnostics};
use locale::tr;
use parser::Error;

fn error_message(err: Error) -> String {
  use Error::*;
  match err {
    Expected { expected, actual } => tr().diagnostic_expected_got(expected, actual),
  }
}

pub async fn document_diagnostics(ctx: QueryCtx, id: FileId) -> Vec<LspDiagnostic> {
  let file = ctx.get(id);
  let index = file.index();
  GetDiagnostics(id)
    .execute(ctx)
    .await
    .iter()
    .map(|diag| {
      let range = span_to_lsp_range(index, diag.range);
      let mut diag = LspDiagnostic::new_simple(range, error_message(diag.error));
      diag.severity = Some(DiagnosticSeverity::ERROR);
      diag.source = Some("autolang".into());
      diag
    })
    .collect()
}
