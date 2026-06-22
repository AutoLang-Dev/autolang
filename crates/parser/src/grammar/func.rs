use super::prelude::*;

pub fn parameter_list(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T!['(']);

  while !p.at_eof() && !p.at(T![')']) {
    parameter(p);
    if !p.bump_if(T![,]) {
      break;
    }
  }

  p.expect(T![')']);
  p.complete(m, ParameterList)
}

fn parameter(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  pat::pattern(p);
  if p.bump_if(T![:]) {
    types::type_(p);
  }
  p.complete(m, Parameter)
}
