use parser::Error;
use syntax::{DiagPayload, Red};
use text_size::TextRange;

pub struct Diagnostic {
  pub error: Error,
  pub range: TextRange,
}

pub fn collect_diag(diags: &mut Vec<Diagnostic>, tree: &Red) {
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
