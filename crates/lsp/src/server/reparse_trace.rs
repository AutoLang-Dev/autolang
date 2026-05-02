use lsp_types::{Range, Uri};
use serde_json::{Value, json};
use syntax::SyntaxKind;

pub const REPARSE_TRACE_NOTIFICATION: &str = "autolang/reparseTrace";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReparseTrace {
  Noop {
    uri: Uri,
    edit_range: Range,
  },
  Token {
    uri: Uri,
    edit_range: Range,
    dirty: TraceDirtyRange,
  },
  Node {
    uri: Uri,
    edit_range: Range,
    dirty: TraceDirtyRange,
    reparser: String,
    old_kind: SyntaxKind,
    new_kind: SyntaxKind,
  },
  Full {
    uri: Uri,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceDirtyRange {
  pub old: Range,
  pub new: Range,
}

impl ReparseTrace {
  pub fn noop(uri: Uri, edit_range: Range) -> Self {
    Self::Noop { uri, edit_range }
  }

  pub fn token(uri: Uri, edit_range: Range, dirty: TraceDirtyRange) -> Self {
    Self::Token {
      uri,
      edit_range,
      dirty,
    }
  }

  pub fn node(
    uri: Uri,
    edit_range: Range,
    dirty: TraceDirtyRange,
    reparser: String,
    old_kind: SyntaxKind,
    new_kind: SyntaxKind,
  ) -> Self {
    Self::Node {
      uri,
      edit_range,
      dirty,
      reparser,
      old_kind,
      new_kind,
    }
  }

  pub fn full(uri: Uri) -> Self {
    Self::Full { uri }
  }

  pub fn to_json(&self) -> Value {
    match self {
      ReparseTrace::Noop { uri, edit_range } => json!({
        "uri": uri.to_string(),
        "strategy": "noop",
        "editRange": edit_range,
      }),
      ReparseTrace::Token {
        uri,
        edit_range,
        dirty,
      } => json!({
        "uri": uri.to_string(),
        "strategy": "token",
        "editRange": edit_range,
        "dirtyRange": dirty_range(*dirty),
      }),
      ReparseTrace::Node {
        uri,
        edit_range,
        dirty,
        reparser,
        old_kind,
        new_kind,
      } => json!({
        "uri": uri.to_string(),
        "strategy": "node",
        "editRange": edit_range,
        "dirtyRange": dirty_range(*dirty),
        "reparser": reparser,
        "oldKind": format!("{old_kind:?}"),
        "newKind": format!("{new_kind:?}"),
      }),
      ReparseTrace::Full { uri } => json!({
        "uri": uri.to_string(),
        "strategy": "full",
      }),
    }
  }
}

fn dirty_range(range: TraceDirtyRange) -> Value {
  json!({
    "old": range.old,
    "new": range.new,
  })
}
