use super::prelude::*;

pub fn type_(p: &mut Parser) -> CompletedMarker {
  match p.current() {
    T![_] => infer_type(p),
    Ident | T![self] | T![super] | T![unit] => path_type(p),
    T![&] => ref_type(p),
    T![*] => ptr_type(p),
    T!['('] => tuple_type(p, true),
    T!['['] => array_or_slice_type(p),
    T!['{'] => record_type(p, true),
    _ => error_type(p),
  }
}

fn infer_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![_]);
  p.complete(m, InferType)
}

fn path_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  paths::path(p);
  p.complete(m, PathType)
}

fn ref_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![&]);
  p.bump_if(T![mut]);
  types::type_(p);
  p.complete(m, RefType)
}

fn ptr_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![*]);
  p.bump_if(T![mut]);
  types::type_(p);
  p.complete(m, PtrType)
}

pub fn tuple_type(p: &mut Parser, allow_fn: bool) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['(']);
  while !p.at_eof() && !p.at(T![')']) {
    type_field(p, false);

    if !p.bump_if(T![,]) {
      break;
    }
  }
  p.expect(T![')']);

  let completed = p.complete(m, TupleType);

  if allow_fn && (p.at(T![mut]) || p.at(T![->])) {
    let m = p.precede(completed);
    p.bump_if(T![mut]);
    if p.expect(T![->]) {
      types::type_(p);
    }
    p.complete(m, FnType)
  } else {
    completed
  }
}

pub fn array_or_slice_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T!['[']);
  types::type_(p);
  let kind = if p.bump_if(T![;]) {
    expr::expr(p);
    ArrayType
  } else {
    SliceType
  };
  p.expect(T![']']);
  p.complete(m, kind)
}

pub fn record_type(p: &mut Parser, allow_fn: bool) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['{']);
  while !p.at_eof() && !p.at(T!['}']) {
    type_field(p, true);
    if !p.bump_if(T![,]) {
      break;
    }
  }
  p.expect(T!['}']);

  let completed = p.complete(m, RecordType);

  if allow_fn && (p.at(T![mut]) || p.at(T![->])) {
    let m = p.precede(completed);
    p.bump_if(T![mut]);
    if p.expect(T![->]) {
      types::type_(p);
    }
    p.complete(m, FnType)
  } else {
    completed
  }
}

fn type_field(p: &mut Parser, record: bool) -> CompletedMarker {
  let m = p.start();
  attrs::attrs(p);
  attrs::visibility(p);
  if record || (p.at(Ident) && p.nth_at(1, T![:])) {
    field_name(p);
    p.expect(T![:]);
  }
  types::type_(p);
  p.complete(m, TypeField)
}

pub fn field_name(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  if !p.bump_if(Int) {
    expect_ident(p);
  }
  p.complete(m, FieldName)
}

pub fn fn_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  match p.current() {
    T!['('] => tuple_type(p, false),
    T!['{'] => record_type(p, false),
    kind => {
      p.error(Error::Expected {
        expected: T!['('],
        actual: kind,
      });
      error_type(p)
    }
  };

  p.bump_if(T![mut]);
  p.expect(T![->]);
  types::type_(p);

  p.complete(m, FnType)
}

fn error_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.error(Error::Expected {
    expected: Ident,
    actual: p.current(),
  });
  if !at_type_recovery(p) {
    p.bump_any();
  }
  p.complete(m, ErrorType)
}

fn at_type_recovery(p: &Parser) -> bool {
  p.at_eof() || p.at(T![;]) || p.at(T![,]) || p.at(T![')']) || p.at(T!['}'])
}
