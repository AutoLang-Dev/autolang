mod diagnostics;
mod document;
mod document_symbols;
mod reparse_trace;
mod semantic_tokens;

#[cfg(test)]
pub(crate) use semantic_tokens::TokenType;

pub use reparse_trace::{REPARSE_TRACE_NOTIFICATION, ReparseTrace};

use crate::server::semantic_tokens::{SemanticCache, SemanticDirty, tokens_legend};
use line_index::LineIndex;
use lsp_types::*;
use serde_json::{Value, json};
use std::collections::HashMap;

pub const SYNTAX_TREE_REQUEST: &str = "autolang/syntaxTree";

pub struct Document {
  pub text: String,
  pub line_index: LineIndex,
  pub parse: syntax::Parse,
  pub semantic_cache: SemanticCache,
  pub semantic_dirty: SemanticDirty,
}

pub struct Server {
  documents: HashMap<Uri, Document>,
}

impl Server {
  pub fn new(_: InitializeParams) -> Self {
    Self {
      documents: HashMap::new(),
    }
  }

  pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
      text_document_sync: Some(TextDocumentSyncCapability::Options(
        TextDocumentSyncOptions {
          open_close: Some(true),
          change: Some(TextDocumentSyncKind::INCREMENTAL),
          save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
            include_text: Some(true),
          })),
          ..Default::default()
        },
      )),
      semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
        SemanticTokensOptions {
          legend: tokens_legend(),
          full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
          ..Default::default()
        },
      )),
      document_symbol_provider: Some(OneOf::Left(true)),
      ..Default::default()
    }
  }
}

pub fn syntax_tree_uri(params: Value) -> Option<Uri> {
  params
    .get("textDocument")?
    .get("uri")?
    .as_str()?
    .parse()
    .ok()
}

pub fn syntax_tree_result(tree: String) -> Value {
  json!({ "tree": tree })
}
