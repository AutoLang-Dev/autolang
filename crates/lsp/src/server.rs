mod document;
mod semantic_tokens;

use crate::server::{document::Document, semantic_tokens::tokens_legend};
use lsp_types::*;
use std::collections::HashMap;

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
      text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
      semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
        SemanticTokensOptions {
          legend: tokens_legend(),
          full: Some(SemanticTokensFullOptions::Delta { delta: Some(false) }),
          ..Default::default()
        },
      )),
      ..Default::default()
    }
  }
}
