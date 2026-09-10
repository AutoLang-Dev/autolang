use super::prelude::*;

pub fn pattern(p: &mut Parser) -> CompletedMarker {
  match p.current() {
    T![_] => wildcard_pat(p),
    T![mut] | Ident => ident_pat(p),
    T!['('] => tuple_pat(p),
    T!['{'] => record_pat(p),
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

pub fn tuple_pat(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T!['(']);

  while !p.at_eof() && !p.at(T![')']) {
    pat_field(p, false);

    if !p.bump_if(T![,]) {
      break;
    }
  }

  p.expect(T![')']);
  p.complete(m, TuplePat)
}

pub fn record_pat(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T!['{']);

  while !p.at_eof() && !p.at(T!['}']) {
    pat_field(p, true);

    if !p.bump_if(T![,]) {
      break;
    }
  }

  p.expect(T!['}']);
  p.complete(m, RecordPat)
}

fn pat_field(p: &mut Parser, record: bool) -> CompletedMarker {
  let m = p.start();
  if record {
    expect_name(p);
    if p.bump_if(T![:]) {
      pattern(p);
    }
  } else {
    if p.at(Ident) && p.nth_at(1, T![:]) {
      expect_name(p);
      p.expect(T![:]);
    }
    pattern(p);
  }
  p.complete(m, PatField)
}

fn error_pat(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  expect_name(p);
  if !p.at(T![:]) && !p.at_eof() {
    p.bump_any();
  }
  p.complete(m, ErrorPat)
}
