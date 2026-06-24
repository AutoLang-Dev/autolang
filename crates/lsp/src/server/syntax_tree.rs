use crate::server::{Server, document::Document};
use lsp_types::{TextDocumentIdentifier, Uri, notification::Notification};
use rgt::red::WalkEvent;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Write};
use syntax::DiagPayload;

#[derive(Debug)]
pub enum SyntaxTreeRequest {}

impl Notification for SyntaxTreeRequest {
  type Params = SyntaxTreeParams;
  const METHOD: &'static str = "autolang/syntaxTree";
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeParams {
  pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTree {
  pub tree: String,
}

fn dump_syntax_tree(doc: &Document) -> Result<String, fmt::Error> {
  let mut indent = "".to_string();
  let mut buf = "".to_string();

  for event in doc.syntax_tree().preorder() {
    match event {
      WalkEvent::Enter(red) => {
        let kind = red.kind();

        let range = red.range();
        let start: u32 = range.start().into();
        let end: u32 = range.end().into();

        write!(&mut buf, "{indent}{kind:?}@{start}..{end}")?;

        if red.is_token() {
          let text = doc.text_of(range);
          write!(&mut buf, " {text:?}")?;

          if let Some(DiagPayload::Diag(error)) = &red.payload().diag {
            writeln!(&mut buf)?;
            write!(&mut buf, "{indent}  diag: {error:?}")?;
          }
        }

        writeln!(&mut buf)?;
        indent += "  ";
      }

      WalkEvent::Leave(_) => {
        indent.pop();
        indent.pop();
      }
    }
  }

  Ok(buf)
}

impl Server {
  pub fn syntax_tree(&self, uri: &Uri) -> Option<SyntaxTree> {
    let doc = self.get_document(uri)?;

    let tree = dump_syntax_tree(doc).ok()?;

    Some(SyntaxTree { tree })
  }
}
