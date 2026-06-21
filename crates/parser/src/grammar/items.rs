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
    BindingItem => {
      pat::pattern(p);
      p.expect(T![:]);
      binding_item_body(p);
    }
    _ => {
      expect_ident(p);
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
    T![mut] => BindingItem,
    Ident | T![_] if nth_at_single_colon(p, 1) => match p.nth(2) {
      T![mod] => ModuleItem,
      T![fn] => FunctionItem,
      T![type] | T![nominal] => TypeItem,
      _ => BindingItem,
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
  p.expect(T![fn]);

  if p.at(T!['(']) {
    func::parameter_list(p);
  }
  p.bump_if(T![mut]);
  if p.bump_if(T![->]) {
    types::type_(p);
  }
  if p.bump_if(T![=]) {
    expr::expr(p);
  }
}

fn type_item_body(p: &mut Parser) {
  let kind = p.start();
  if p.at(T![nominal]) {
    p.bump(T![nominal]);
  } else {
    p.expect(T![type]);
  }
  p.complete(kind, TypeKind);

  if p.expect(T![=]) {
    types::type_(p);
  }
}

fn binding_item_body(p: &mut Parser) {
  if !p.at(T![=]) {
    types::type_(p);
  }
  if p.bump_if(T![=]) {
    expr::expr(p);
  }
}

fn using_item_body(p: &mut Parser) {
  p.expect(T![using]);
  using_tree(p);
}

fn using_tree(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  match p.current() {
    T![_] => p.bump(T![_]),
    T![*] => {
      p.error(Error::Expected {
        expected: T![_],
        actual: T![*],
      });
      p.bump_any();
    }
    T!['{'] => {
      using_tree_list(p);
    }
    Ident | T![self] | T![super] | T![unit] | T![:] => {
      if nth_at_single_colon(p, 0) {
        p.error(Error::Expected {
          expected: T![::],
          actual: T![:],
        });
        p.bump_any();
      }

      paths::path_allow_trailing_colon_colon(p);

      match p.current() {
        T!['{'] => {
          using_tree_list(p);
        }
        T![_] => p.bump(T![_]),
        T![*] => {
          p.error(Error::Expected {
            expected: T![_],
            actual: T![*],
          });
          p.bump_any();
        }
        _ => (),
      }

      if p.at(T![as]) {
        rename(p);
      }
    }
    _ => {
      expect_ident(p);
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
  expect_ident(p);
  p.complete(m, Rename)
}

fn at_item_recovery(p: &Parser) -> bool {
  p.at_eof() || p.at(T![;]) || p.at(T!['}'])
}
