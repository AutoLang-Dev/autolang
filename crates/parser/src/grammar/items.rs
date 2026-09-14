use super::prelude::*;

// skip attrs if marker is Some(...)
pub fn item(p: &mut Parser, marker: Option<Marker>) -> CompletedMarker {
  let m = marker.unwrap_or_else(|| {
    let m = p.start();
    attrs::attrs(p);
    m
  });

  if p.bump_if(T![;]) {
    return p.complete(m, EmptyItem);
  }

  attrs::visibility(p);

  let kind = item_kind_after_prefix(p);

  match kind {
    ErrorItem => {
      p.error(Error::Expected {
        expected: Ident,
        actual: p.current(),
      });
      if !at_item_recovery(p) {
        p.bump_any();
      }
    }
    UsingItem => using_item_body(p),
    _ => {
      name(p);
      p.expect(T![:]);
      match kind {
        ModuleItem => module_item_body(p),
        FunctionItem => function_item_body(p),
        TypeItem => type_item_body(p),
        _ => unreachable!(),
      }
    }
  }
  p.expect(T![;]);

  p.complete(m, kind)
}

fn item_kind_after_prefix(p: &Parser) -> SyntaxKind {
  match p.current() {
    T![using] => UsingItem,
    Ident if nth_at_single_colon(p, 1) => match p.nth(2) {
      T![mod] => ModuleItem,
      T!['('] | T!['{'] => FunctionItem,
      T![type] | T![nominal] => TypeItem,
      _ => ErrorItem,
    },
    _ => ErrorItem,
  }
}

fn module_item_body(p: &mut Parser) {
  p.expect(T![mod]);

  if p.bump_if(T![=]) {
    module(p);
  }
}

pub fn module(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  if p.expect(T!['{']) {
    module_inner(p, true);
    p.expect(T!['}']);
  }
  p.complete(m, Module)
}

fn function_item_body(p: &mut Parser) {
  types::fn_type(p);
  if p.bump_if(T![=]) {
    expr::expr(p);
  }
}

fn type_item_body(p: &mut Parser) {
  let kind = p.start();
  let nominal = p.at(T![nominal]);
  p.bump_any();
  p.complete(kind, TypeKind);

  if p.expect(T![=]) {
    types::nominal_type(p, nominal);
  }
}

fn using_item_body(p: &mut Parser) {
  p.expect(T![using]);
  using_unit_name(p);
  p.expect(T![::]);
  using_tree(p);
}

fn using_unit_name(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  match p.current() {
    Ident => {
      name(p);
    }
    T![unit] => {
      p.bump(T![unit]);
    }
    _ => {
      p.expect(Ident);
    }
  }

  p.complete(m, UsingUnitName)
}

pub fn using_tree(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  match p.current() {
    Ident => {
      name(p);
      if p.bump_if(T![::]) {
        using_tree(p);
      } else if p.at(T![as]) {
        rename(p);
      }
    }
    T![_] => {
      p.bump(T![_]);
    }
    T!['{'] => {
      using_tree_list(p);
    }
    T![*] => {
      p.error(Error::Expected {
        expected: T![_],
        actual: T![*],
      });
      p.bump(T![*]);
    }
    _ => {
      p.expect(Ident);
    }
  }

  p.complete(m, UsingTree)
}

pub fn using_tree_list(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T!['{']);

  while !p.at_eof() && !p.at(T!['}']) {
    using_tree(p);
    if !p.bump_if(T![,]) {
      break;
    }
  }

  p.expect(T!['}']);
  p.complete(m, UsingTreeList)
}

fn rename(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![as]);
  name(p);
  p.complete(m, Rename)
}

fn at_item_recovery(p: &Parser) -> bool {
  p.at_eof() || p.at(T![;]) || p.at(T!['}'])
}
