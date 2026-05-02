use crate::{GreenNode, Lang, SyntaxNode};
use rowan::{GreenNodeBuilder, Language};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parse {
  pub(crate) green: GreenNode,
  pub(crate) errors: Vec<SyntaxError>,
}

impl Parse {
  pub fn syntax_node(&self) -> SyntaxNode {
    SyntaxNode::new_root(self.green.clone())
  }

  pub fn green_node(&self) -> &GreenNode {
    &self.green
  }

  pub fn errors(&self) -> &[SyntaxError] {
    &self.errors
  }

  pub fn into_parts(self) -> (GreenNode, Vec<SyntaxError>) {
    (self.green, self.errors)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxError {
  pub error: parser::Error,
  pub offset: u32,
}

pub fn parse(text: &str) -> Parse {
  let lexed = parser::LexedStr::lex(text);
  let output = parser::parse_lexed(&lexed);
  let (parse, is_eof) = parse_output(&lexed, &output);

  assert!(
    is_eof,
    "parser output did not consume all non-trivia tokens"
  );

  parse
}

pub(crate) fn parse_fragment(
  lexed: &parser::LexedStr<'_>,
  output: &parser::Output,
) -> Option<Parse> {
  let (parse, is_eof) = parse_output(lexed, output);
  is_eof.then_some(parse)
}

fn parse_output(lexed: &parser::LexedStr<'_>, output: &parser::Output) -> (Parse, bool) {
  let mut builder = GreenNodeBuilder::new();
  let mut errors = Vec::new();

  let is_eof = parser::emit_tree_steps(lexed, output, &mut |step| match step {
    parser::TreeStep::Enter(kind) => builder.start_node(Lang::kind_to_raw(kind)),
    parser::TreeStep::Exit => builder.finish_node(),
    parser::TreeStep::Token { kind, text } => builder.token(Lang::kind_to_raw(kind), text),
    parser::TreeStep::Error { error, pos } => errors.push(SyntaxError { error, offset: pos }),
  });

  (
    Parse {
      green: builder.finish(),
      errors,
    },
    is_eof,
  )
}
