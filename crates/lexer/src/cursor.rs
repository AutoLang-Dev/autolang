use std::str::Chars;

use crate::tokens::{
  Token,
  TokenKind::{self, *},
};

const EOF_CHAR: char = '\0';

fn is_start(c: char) -> bool {
  c == '_' || unicode_ident::is_xid_start(c)
}

fn is_continue(c: char) -> bool {
  unicode_ident::is_xid_continue(c)
}

pub fn is_whitespace(c: char) -> bool {
  // Pattern_White_Space
  matches!(
    c,
    // End-of-line characters
    | '\u{000A}' // line feed
    | '\u{000B}' // vertical tab
    | '\u{000C}' // form feed
    | '\u{000D}' // carriage return
    | '\u{0085}' // next line
    | '\u{2028}' // line separator
    | '\u{2029}' // paragraph separator

    // `Default_Ignorable_Code_Point`
    | '\u{200E}' // LTR
    | '\u{200F}' // RTL

    | '\u{0009}' // tab
    | '\u{0020}' // space
  )
}

pub fn strip_shebang(input: &str) -> usize {
  if !input.starts_with("#!") {
    return 0;
  }

  input.lines().next().unwrap_or_default().len()
}

pub struct Cursor<'a> {
  len_remaining: usize,
  chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
  pub fn new(input: &'a str) -> Self {
    Cursor {
      len_remaining: input.len(),
      chars: input.chars(),
    }
  }

  pub fn peek(&self, n: usize) -> char {
    self.chars.clone().nth(n).unwrap_or(EOF_CHAR)
  }

  pub fn first(&self) -> char {
    self.peek(0)
  }

  pub fn second(&self) -> char {
    self.peek(1)
  }

  pub fn third(&self) -> char {
    self.peek(2)
  }

  pub fn is_eof(&self) -> bool {
    self.chars.as_str().is_empty()
  }

  pub fn pos_within_token(&self) -> u32 {
    (self.len_remaining - self.chars.as_str().len()) as u32
  }

  pub fn reset_pos_within_token(&mut self) {
    self.len_remaining = self.chars.as_str().len();
  }

  pub fn bump(&mut self) -> Option<char> {
    self.chars.next()
  }

  pub fn eat_while(&mut self, pred: impl Fn(char) -> bool) {
    while pred(self.first()) && !self.is_eof() {
      self.bump();
    }
  }

  pub fn eat_line(&mut self) {
    self.eat_while(|c| c != '\n');
  }

  pub fn sync(&mut self, kind: TokenKind) -> Token {
    let rest = self.chars.as_str().len();
    let len = (self.len_remaining - rest) as u32;
    self.len_remaining = rest;
    Token::new(kind, len)
  }

  pub fn advance_token(&mut self) -> Token {
    let Some(first) = self.bump() else {
      return Token::new(Eof, 0);
    };

    let kind = match first {
      '/' => match self.first() {
        '/' => {
          self.eat_line();
          Comment
        }
        _ => Slash,
      },

      c if is_whitespace(c) => {
        self.eat_while(is_whitespace);
        Whitespace
      }

      'b' => match self.first() {
        '\'' => {
          self.bump();
          self.single_quoted();
          Literal
        }
        '"' => {
          self.bump();
          self.double_quoted();
          Literal
        }
        _ => self.ident(),
      },

      c if is_start(c) => self.ident(),

      c @ '0'..='9' => {
        self.number(c);
        Literal
      }

      ';' => Semi,
      ',' => Comma,
      '.' => Dot,
      '(' => OpenParen,
      ')' => CloseParen,
      '{' => OpenBrace,
      '}' => CloseBrace,
      '[' => OpenBracket,
      ']' => CloseBracket,
      '@' => At,
      '#' => Hash,
      '~' => Tilde,
      '?' => Question,
      ':' => Colon,
      '$' => Dollar,
      '=' => Eq,
      '!' => Bang,
      '<' => Lt,
      '>' => Gt,
      '-' => Minus,
      '&' => Amp,
      '|' => Pipe,
      '+' => Plus,
      '*' => Star,
      '^' => Caret,
      '%' => Percent,
      '\\' => Backslash,

      '\'' => match self.first() {
        '\'' => {
          self.eat_line();
          Literal
        }
        _ => self.label_or_char(),
      },

      '"' => {
        self.double_quoted();
        Literal
      }

      _ => Unknown,
    };

    self.sync(kind)
  }

  fn single_quoted(&mut self) {
    // check if it's one-symbol
    if self.second() == '\'' && self.first() != '\\' {
      self.bump();
      self.bump();
      return;
    }

    // more than one-symbol
    loop {
      match self.first() {
        '\'' => {
          self.bump();
          return;
        }

        // probably beginning of the comment
        '/' if self.second() == '/' => break,

        '\n' => break,

        EOF_CHAR if self.is_eof() => break,

        '\\' => {
          self.bump();
          self.bump();
        }

        _ => {
          self.bump();
        }
      }
    }
  }

  fn double_quoted(&mut self) {
    while let Some(c) = self.bump() {
      match c {
        '\"' | '\n' => break,
        '\\' if self.first() == '\\' || self.first() == '"' => {
          self.bump();
        }
        _ => (),
      }
    }
  }

  fn label_or_char(&mut self) -> TokenKind {
    let starts_with_number = self.first().is_ascii_digit();
    let can_be_a_label = if self.second() == '\'' {
      false
    } else {
      is_start(self.first()) || starts_with_number
    };

    if !can_be_a_label {
      self.single_quoted();
      return Literal;
    }

    self.bump();
    self.eat_while(is_continue);

    match self.first() {
      '\'' => {
        self.bump();
        Literal
      }
      _ => Label,
    }
  }

  fn ident(&mut self) -> TokenKind {
    self.eat_while(is_continue);

    match self.first() {
      '\'' | '"' => UnknownPrefix,
      _ => Ident,
    }
  }

  fn number(&mut self, first: char) {
    let mut radix = 10;
    if first == '0' {
      radix = match self.first() {
        'b' => 2,
        'o' => 8,
        'x' => 16,
        'r' => match self.second() {
          radix_char @ '0'..='z' => {
            self.bump();
            radix_char.to_digit(36).unwrap()
          }
          _ => return,
        },
        '0'..='9' | '\'' => 10,
        _ => return,
      };
      self.bump();
    }

    loop {
      match self.first() {
        '\'' => {
          self.bump();
        }
        digit if digit.is_digit(radix) => {
          self.bump();
        }
        _ => break,
      }
    }
  }
}
