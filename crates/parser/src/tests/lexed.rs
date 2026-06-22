use crate::{SyntaxKind, infra::lexed::LexedStr};
use std::fmt::Debug;

#[derive(PartialEq, Eq)]
struct RawToken<'text> {
  kind: SyntaxKind,
  text: &'text str,
  offset: u32,
}

impl Debug for RawToken<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}({:?})@{}", self.kind, self.text, self.offset)
  }
}

fn raw_tokens<'text>(lexed: &LexedStr<'text>) -> Vec<RawToken<'text>> {
  (0..lexed.len())
    .map(|raw| RawToken {
      kind: lexed.kind(raw),
      text: lexed.range_text(raw..raw + 1),
      offset: lexed.text_start(raw),
    })
    .collect()
}

macro_rules! lex_snap {
  ($input:literal) => {{
    let input = $input;
    let lexed = LexedStr::new(input);
    snap!(input, raw_tokens(&lexed));
  }};
}

#[test]
fn converter_classifies_keywords_and_identifiers() {
  lex_snap!("fn mod true false ident _");
}

#[test]
fn converter_classifies_literals() {
  lex_snap!(r#"42 'a' b'x' "s" ''raw"#);
}

#[test]
fn converter_keeps_shebang_offset() {
  lex_snap!("#!/usr/bin/env autolang\nfoo")
}

#[test]
fn converter_keeps_unknown_tokens_silent() {
  lex_snap!("` abc' def\"");
}
