use super::prelude::*;

pub fn attrs(p: &mut Parser) {
  while p.at(T![#]) {
    attr(p);
  }
}

pub fn visibility(p: &mut Parser) -> Option<CompletedMarker> {
  if !at_visibility(p) {
    return None;
  }

  let m = p.start();
  p.bump(p.current());
  Some(p.complete(m, Visibility))
}

pub fn nth_visibility(p: &Parser, n: usize) -> bool {
  matches!(p.nth(n), T![pub] | T![pro] | T![pri])
}

pub fn attr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![#]);
  if p.at(T![!]) {
    p.error(Error::Expected {
      expected: T!['['],
      actual: T![!],
    });
    p.bump_any();
  }
  attr_inner(p);
  p.complete(m, Attr)
}

pub fn attr_inner(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T!['[']);

  while !p.at_eof() && !p.at(T![']']) {
    attr_item(p);
    if !p.bump_if(T![,]) {
      break;
    }
  }

  p.expect(T![']']);
  p.complete(m, AttrInner)
}

fn attr_item(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  paths::path(p);
  if p.at(T![=]) || matches!(p.current(), T!['('] | T!['['] | T!['{']) {
    attr_arg(p);
  }
  p.complete(m, AttrItem)
}

pub fn attr_arg(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  if p.bump_if(T![=]) {
    expr::expr(p);
  } else {
    token_trees::delimited_token_tree(p);
  }
  p.complete(m, AttrArg)
}

pub fn at_visibility(p: &Parser) -> bool {
  nth_visibility(p, 0)
}
