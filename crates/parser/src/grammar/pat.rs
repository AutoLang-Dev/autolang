use super::prelude::*;

pub fn pattern(p: &mut Parser) -> CompletedMarker {
  match p.current() {
    T![_] => wildcard_pat(p),
    T![mut] | Ident => ident_pat(p),
    _ => error_pat(p),
  }
}

fn wildcard_pat(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![_]);
  p.complete(m, WildcardPat)
}

fn ident_pat(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump_if(T![mut]);
  expect_name(p);
  p.complete(m, IdentPat)
}

fn error_pat(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  expect_name(p);
  if !p.at(T![:]) && !p.at_eof() {
    p.bump_any();
  }
  p.complete(m, ErrorPat)
}
