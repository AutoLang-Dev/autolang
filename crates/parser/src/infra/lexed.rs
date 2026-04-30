use crate::{
  SyntaxKind::{self, *},
  infra::input::Input,
};
use lexer::{Token, TokenKind as TK};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct LexedStr<'a> {
  text: &'a str,
  kind: Vec<SyntaxKind>,
  start: Vec<u32>,
}

impl<'a> LexedStr<'a> {
  pub fn new(text: &'a str, allow_shebang: bool) -> Self {
    let mut converter = Converter::new(text);
    let mut offset = 0;

    if allow_shebang {
      let shebang_len = lexer::strip_shebang(text);
      if shebang_len != 0 {
        converter.push(Shebang, offset);
        offset += shebang_len;
      }
    }

    converter.convert(offset)
  }

  pub fn lex(text: &'a str) -> Self {
    Self::new(text, true)
  }

  pub fn relex(text: &'a str) -> Self {
    Self::new(text, false)
  }

  pub fn range_text(&self, range: Range<u32>) -> &'a str {
    assert!(range.start < range.end);
    assert!(range.end <= self.len());
    let start = self.text_start(range.start) as usize;
    let end = self.text_start(range.end) as usize;
    &self.text[start..end]
  }

  pub fn text_start(&self, raw: u32) -> u32 {
    let raw = raw as usize;
    assert!(raw < self.start.len());
    self.start[raw]
  }

  pub fn to_input(&self) -> Input {
    let mut input = Input::with_capacity(self.len() as usize);
    let mut prev_input = false;
    let mut prev_raw_end = 0;

    for raw in 0..self.len() {
      let kind = self.kind(raw);
      if kind.is_trivia() {
        continue;
      }

      let raw_start = self.text_start(raw);
      if prev_input && prev_raw_end == raw_start {
        input.was_joint();
      }

      input.push(kind);
      prev_input = true;
      prev_raw_end = self.text_start(raw + 1);
    }

    input
  }

  pub fn kind(&self, raw: u32) -> SyntaxKind {
    self.kind.get(raw as usize).copied().unwrap_or(Eof)
  }

  pub fn len(&self) -> u32 {
    (self.kind.len() - 1)
      .try_into()
      .expect("too many raw tokens")
  }

  pub fn is_empty(&self) -> bool {
    let len = self.kind.len();
    assert_ne!(len, 0);
    len == 1
  }
}

#[derive(Debug)]
struct Converter<'a> {
  text: &'a str,
  kind: Vec<SyntaxKind>,
  start: Vec<u32>,
}

impl<'a> Converter<'a> {
  fn new(text: &'a str) -> Self {
    Self {
      text,
      kind: Vec::new(),
      start: Vec::new(),
    }
  }

  fn convert(mut self, mut offset: usize) -> LexedStr<'a> {
    for token in lexer::lex(&self.text[offset..]) {
      self.extend_token(token, offset);
      offset += token.len as usize;
    }

    assert_eq!(offset, self.text.len());
    self.push(Eof, offset);

    LexedStr {
      text: self.text,
      kind: self.kind,
      start: self.start,
    }
  }

  fn push(&mut self, kind: SyntaxKind, offset: usize) {
    let offset = offset.try_into().expect("source text is too large");
    self.kind.push(kind);
    self.start.push(offset);
  }

  fn extend_token(&mut self, token: Token, offset: usize) {
    let end = offset + token.len as usize;
    let text = &self.text[offset..end];
    let kind = self.convert_kind(token.kind, text);
    self.push(kind, offset);
  }

  fn convert_kind(&mut self, kind: TK, text: &str) -> SyntaxKind {
    match kind {
      TK::Whitespace => Whitespace,
      TK::Comment => Comment,
      TK::Ident => classify_ident(text),
      TK::UnknownPrefix => UnknownPrefix,
      TK::Label => Label,
      TK::Literal => classify_literal(text),
      TK::Semi => Semi,
      TK::Comma => Comma,
      TK::Dot => Dot,
      TK::OpenParen => OpenParen,
      TK::CloseParen => CloseParen,
      TK::OpenBrace => OpenBrace,
      TK::CloseBrace => CloseBrace,
      TK::OpenBracket => OpenBrack,
      TK::CloseBracket => CloseBrack,
      TK::At => At,
      TK::Hash => Hash,
      TK::Tilde => Tilde,
      TK::Question => Question,
      TK::Colon => Colon,
      TK::Dollar => Dollar,
      TK::Eq => Eq,
      TK::Bang => Bang,
      TK::Lt => Lt,
      TK::Gt => Gt,
      TK::Minus => Minus,
      TK::Amp => Amp,
      TK::Pipe => Pipe,
      TK::Plus => Plus,
      TK::Star => Star,
      TK::Slash => Slash,
      TK::Backslash => Backslash,
      TK::Caret => Caret,
      TK::Percent => Percent,
      TK::Unknown => Unknown,
      TK::Eof => unreachable!("lexer::lex does not yield EOF tokens"),
    }
  }
}

fn classify_ident(text: &str) -> SyntaxKind {
  match text {
    "_" => Underscore,
    text => SyntaxKind::from_keyword(text).unwrap_or(Ident),
  }
}

fn classify_literal(text: &str) -> SyntaxKind {
  match text.as_bytes() {
    [b'0'..=b'9', ..] => Int,
    [b'\'', b'\'', ..] => RawString,
    [b'\'', ..] => Char,
    [b'b', b'\'', ..] => Byte,
    [b'b', b'"', ..] | [b'"', ..] => String,
    _ => String,
  }
}
