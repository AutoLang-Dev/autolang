use async_inc::QueryCtx;
use async_lsp::lsp_types::{TextDocumentIdentifier, request::Request};
use ide::FileId;
use rgt::red::WalkEvent;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use syntax::DiagPayload;

#[derive(Debug)]
pub enum SyntaxTreeRequest {}

impl Request for SyntaxTreeRequest {
  type Params = SyntaxTreeParams;
  type Result = SyntaxTreeResponse;
  const METHOD: &'static str = "autolang/syntaxTree";
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeParams {
  pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeResponse {
  pub tree: String,
}

pub async fn syntax_tree(ctx: QueryCtx, id: FileId) -> SyntaxTreeResponse {
  let file = ctx.get(id);

  let mut indent = "".to_string();
  let mut buf = "".to_string();

  for event in file.red_tree().preorder() {
    match event {
      WalkEvent::Enter(red) => {
        let kind = red.kind();

        let range = red.range();
        let start: u32 = range.start().into();
        let end: u32 = range.end().into();

        write!(&mut buf, "{indent}{kind:?}@{start}..{end}").unwrap();

        if red.is_token() {
          let text = file.text_of(range);
          write!(&mut buf, " {text:?}").unwrap();

          if let Some(DiagPayload::Diag(error)) = &red.payload().diag {
            writeln!(&mut buf).unwrap();
            write!(&mut buf, "{indent}  diag: {error:?}").unwrap();
          }
        }

        writeln!(&mut buf).unwrap();
        indent += "  ";
      }

      WalkEvent::Leave(_) => {
        indent.pop();
        indent.pop();
      }
    }
  }

  SyntaxTreeResponse { tree: buf }
}
