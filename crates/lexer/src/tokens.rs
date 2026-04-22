use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
  Whitespace,
  Comment,
  Ident,
  UnknownPrefix,
  Label,
  Literal,
  Semi,
  Comma,
  Dot,
  OpenParen,
  CloseParen,
  OpenBrace,
  CloseBrace,
  OpenBracket,
  CloseBracket,
  At,
  Hash,
  Tilde,
  Question,
  Colon,
  Dollar,
  Eq,
  Bang,
  Lt,
  Gt,
  Minus,
  Amp,
  Pipe,
  Plus,
  Star,
  Slash,
  Backslash,
  Caret,
  Percent,
  Unknown,
  Eof,
}

impl TokenKind {
  pub fn is_trivia(self) -> bool {
    use TokenKind::*;
    matches!(self, Whitespace | Comment)
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
  pub kind: TokenKind,
  pub len: u32,
}

impl Token {
  pub fn new(kind: TokenKind, len: u32) -> Self {
    Self { kind, len }
  }
}

impl Debug for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}({})", self.kind, self.len)
  }
}
