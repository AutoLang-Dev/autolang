mod cursor;
mod tokens;

pub use cursor::*;
pub use tokens::*;

pub fn lex(input: &str) -> impl Iterator<Item = Token> {
  let mut cursor = Cursor::new(input);
  std::iter::from_fn(move || {
    let token = cursor.advance_token();
    if token.kind != TokenKind::Eof {
      Some(token)
    } else {
      None
    }
  })
}

#[cfg(test)]
mod tests;
