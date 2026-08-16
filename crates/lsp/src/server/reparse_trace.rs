use async_lsp::lsp_types::{Range, TextDocumentIdentifier, Url, notification::Notification};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ReparseTraceNotification {}

impl Notification for ReparseTraceNotification {
  type Params = ReparseTraceParams;
  const METHOD: &'static str = "autolang/reparseTrace";
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReparseTraceParams {
  pub text_document: TextDocumentIdentifier,
  pub ranges: Vec<Range>,
}

impl ReparseTraceParams {
  pub fn new(uri: Url, ranges: Vec<Range>) -> Self {
    Self {
      text_document: TextDocumentIdentifier::new(uri),
      ranges,
    }
  }
}
