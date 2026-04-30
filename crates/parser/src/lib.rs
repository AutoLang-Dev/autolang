mod grammar;
mod infra;

pub use grammar::*;
pub use infra::{
  event::{Error, Output, Step},
  input::Input,
  kinds::*,
  lexed::LexedStr,
  tree_step::{TreeStep, emit_tree_steps},
};

use infra::parser::Parser;

pub fn parse(lexed: &LexedStr<'_>) -> Output {
  let input = lexed.to_input();
  let mut parser = Parser::new(input);
  source_file(&mut parser);
  parser.finish()
}

#[cfg(test)]
mod tests;
