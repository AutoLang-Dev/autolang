use crate::FileId;
use async_inc::Query;
use line_index::TextRange;
use parser::Error;
use syntax::{DiagPayload, Red};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
  pub error: Error,
  pub range: TextRange,
}

fn collect_diag(diags: &mut Vec<Diagnostic>, tree: &Red) {
  let Some(diag) = tree.payload().diag.as_ref() else {
    return;
  };

  match diag {
    DiagPayload::Diag(err) => {
      diags.push(Diagnostic {
        error: *err,
        range: tree.range(),
      });
    }
    _ => {
      for child in tree.children() {
        collect_diag(diags, &child);
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GetDiagnostics(pub FileId);

impl Query for GetDiagnostics {
  type Value = Vec<Diagnostic>;

  async fn compute(self, ctx: async_inc::QueryCtx) -> Self::Value {
    let file = ctx.get(self.0);
    let mut diags = vec![];
    collect_diag(&mut diags, &file.red_tree());
    diags
  }
}
