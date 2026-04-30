use super::prelude::*;

pub fn path(p: &mut Parser) -> CompletedMarker {
  path_impl(p, false)
}

pub fn path_allow_trailing_colon_colon(p: &mut Parser) -> CompletedMarker {
  path_impl(p, true)
}

fn path_impl(p: &mut Parser, allow_trailing_colon_colon: bool) -> CompletedMarker {
  let m = p.start();
  p.bump_if(T![::]);
  path_segment(p);

  while p.bump_if(T![::]) {
    if allow_trailing_colon_colon {
      // for error recovery
      if p.at(T![*]) {
        break;
      }
      if matches!(p.current(), T![_] | T!['{']) {
        break;
      }
    }
    path_segment(p);
  }

  p.complete(m, Path)
}

fn path_segment(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  match p.current() {
    Ident => p.bump_any(),
    T![self] => p.bump(T![self]),
    T![super] => p.bump(T![super]),
    T![unit] => p.bump(T![unit]),
    _ => p.error(Error::Expected {
      expected: Ident,
      actual: p.current(),
    }),
  }
  p.complete(m, PathSegment)
}
