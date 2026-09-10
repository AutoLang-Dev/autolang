mod make;
mod reparse;
mod tree;

use crate::{Green, Red, build_syntax_tree};
use parser::{LexedStr, parse};

fn parse_green(text: &str) -> Green {
  let lexed = LexedStr::new(text);
  let output = parse(&lexed);
  build_syntax_tree(&lexed, &output)
}

fn parse_red(text: &str) -> Red {
  Red::new_root(parse_green(text))
}
