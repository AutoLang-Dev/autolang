mod grammar;
mod infra;

pub use grammar::*;
pub use infra::{
  event::{Error, Output, Step},
  kinds::*,
  lexed::LexedStr,
  tree_step::{TreeStep, emit_tree_steps},
};

use infra::parser::Parser;

pub fn parse(text: &str) -> Output {
  let lexed = LexedStr::lex(text);
  parse_lexed(&lexed)
}

pub fn parse_lexed(lexed: &LexedStr<'_>) -> Output {
  let input = lexed.to_input();
  let mut parser = Parser::new(input);
  source_file(&mut parser);
  parser.finish()
}

#[cfg(test)]
mod tests;
