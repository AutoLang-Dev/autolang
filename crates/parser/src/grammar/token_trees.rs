use super::prelude::*;

pub fn delimited_token_tree(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  let close = match p.current() {
    T!['('] => {
      p.bump(T!['(']);
      T![')']
    }
    T!['['] => {
      p.bump(T!['[']);
      T![']']
    }
    T!['{'] => {
      p.bump(T!['{']);
      T!['}']
    }
    _ => unreachable!(),
  };

  while !p.at_eof() && !p.at(close) {
    token_tree(p);
  }
  p.expect(close);

  p.complete(m, DelimitedTokenTree)
}

pub fn token_tree(p: &mut Parser) -> CompletedMarker {
  if matches!(p.current(), T!['('] | T!['['] | T!['{']) {
    return delimited_token_tree(p);
  }

  let m = p.start();
  p.bump_any();
  p.complete(m, TokenTree)
}
