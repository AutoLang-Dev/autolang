pub mod attrs;
pub mod expr;
pub mod items;
pub mod pat;
pub mod paths;
pub mod stmts;
pub mod token_trees;
pub mod types;

mod prelude {
  pub use super::*;
  pub use crate::{
    SyntaxKind::{self, *},
    T,
    infra::{
      event::Error,
      marker::{CompletedMarker, Marker},
      parser::Parser,
    },
  };
}

use prelude::*;

pub fn source_file(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  module_inner(p, false);
  p.complete(m, SourceFile)
}

fn module_inner(p: &mut Parser, stop_on_close_brace: bool) -> CompletedMarker {
  let m = p.start();

  while !(p.at_eof() || stop_on_close_brace && p.at(T!['}'])) {
    if is_item_start(p) || p.at(T![#]) {
      items::item(p, None);
    } else {
      p.error(Error::Expected {
        expected: Ident,
        actual: p.current(),
      });
      p.bump_any();
    }
  }

  p.complete(m, ModuleInner)
}

/// Consumes one identifier into a `Name` node.
///
/// This is the only place that produces names: item names, binding names,
/// field names, path segments and rename targets all go through here.
///
/// `_` is deliberately not a name. It only appears as a wildcard pattern, an
/// infer type, a wildcard expression, or an explicit placeholder written by
/// the caller (`as _`, `using foo::_`).
fn name(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(Ident);
  p.complete(m, Name)
}

fn is_item_start(p: &Parser) -> bool {
  match p.current() {
    T![using] | T![pub] | T![pro] | T![pri] | T![;] => true,
    Ident => nth_at_single_colon(p, 1),
    _ => false,
  }
}

fn nth_at_single_colon(p: &Parser, n: usize) -> bool {
  p.nth_at(n, T![:]) && !p.nth_at(n, T![::])
}

fn recover_balanced(p: &mut Parser, mut stop: impl FnMut(&Parser) -> bool) {
  let mut expected_closes = Vec::new();

  while !p.at_eof() {
    if expected_closes.is_empty() && stop(p) {
      break;
    }

    match p.current() {
      T!['('] => expected_closes.push(T![')']),
      T!['['] => expected_closes.push(T![']']),
      T!['{'] => expected_closes.push(T!['}']),
      close @ (T![')'] | T![']'] | T!['}']) => {
        if expected_closes.last().copied() == Some(close) {
          expected_closes.pop();
        } else if expected_closes.is_empty() {
          break;
        }
      }
      _ => (),
    }

    p.bump_any();
  }
}
