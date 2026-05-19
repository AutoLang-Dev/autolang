use crate::{DiagPayload, Green, Payload};
use parser::{LexedStr, Output, SyntaxKind, TreeStep, emit_tree_steps};
use rgt::builder::Builder;
use text_size::TextSize;

pub fn build_syntax_tree(lexed: &LexedStr, output: &Output) -> Green {
  let mut builder = Builder::new();

  let is_eof = emit_tree_steps(lexed, output, &mut |step| match step {
    TreeStep::Enter(kind) => builder.start_node(kind),
    TreeStep::Exit => builder.finish_node().unwrap(),
    TreeStep::Token { kind, text } => {
      builder.token(kind, TextSize::of(text), Default::default());
    }
    TreeStep::Error(error) => {
      builder.token(
        SyntaxKind::Error,
        0.into(),
        Payload {
          diag: Some(DiagPayload::Diag(error)),
        },
      );
    }
  });

  assert!(is_eof);
  builder.finish().unwrap()
}
