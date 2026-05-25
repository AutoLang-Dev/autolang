pub mod grammar;
mod infra;

pub use infra::{
  event::{Error, Output, Step},
  input::Input,
  kinds::*,
  lexed::LexedStr,
  parser::Parser,
  tree_step::{TreeStep, emit_tree_steps},
};

pub fn parse(lexed: &LexedStr<'_>) -> Output {
  let input = lexed.to_input();
  let mut parser = Parser::new(input);
  grammar::source_file(&mut parser);
  parser.finish()
}

#[cfg(test)]
mod tests;
