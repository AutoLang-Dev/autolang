use super::prelude::*;

pub fn type_(p: &mut Parser) -> CompletedMarker {
  match p.current() {
    T![_] => infer_type(p),
    Ident | T![self] | T![super] | T![unit] => path_type(p),
    T![&] => ref_type(p),
    T![*] => ptr_type(p),
    T!['('] => tuple_type(p),
    T!['['] => array_or_slice_type(p),
    T!['{'] => record_type(p),
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

pub fn tuple_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['(']);
  while !p.at_eof() && !p.at(T![')']) {
    tuple_field(p);

    if !p.bump_if(T![,]) {
      break;
    }
  }
  p.expect(T![')']);

  let completed = p.complete(m, TupleType);

  if p.at(T![mut]) || p.at(T![->]) {
    let m = p.precede(completed);
    p.bump_if(T![mut]);
    if p.expect(T![->]) {
      types::type_(p);
    }
    p.complete(m, FnPtrType)
  } else {
    completed
  }
}

fn tuple_field(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  attrs::attrs(p);
  attrs::visibility(p);
  types::type_(p);
  p.complete(m, TupleField)
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

pub fn record_type(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['{']);
  while !p.at_eof() && !p.at(T!['}']) {
    type_field(p, true);
    if !p.bump_if(T![,]) {
      break;
    }
  }
  p.expect(T!['}']);

  p.complete(m, RecordType)
}

fn type_field(p: &mut Parser, record: bool) -> CompletedMarker {
  let m = p.start();
  attrs::attrs(p);
  attrs::visibility(p);
  field_name(p);
  if record || p.at(T![:]) {
    p.expect(T![:]);
    types::type_(p);
  }
  p.complete(m, TypeField)
}

pub fn field_name(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  if !p.bump_if(Int) {
    expect_ident(p);
  }
  p.complete(m, FieldName)
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
