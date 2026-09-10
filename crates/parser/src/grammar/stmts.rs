use super::prelude::*;

pub fn stmt(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  attrs::attrs(p);

  match p.current() {
    T![let] => let_stmt(p, m),
    Ident if p.nth_at(1, T![:]) && !p.nth_at(1, T![::]) => short_let_stmt(p, m),
    T![using] => using_stmt(p, m),
    _ => expr_stmt(p, m),
  }
}

/// A statement that starts with an expression. What follows the expression
/// decides which statement it is.
fn expr_stmt(p: &mut Parser, m: Marker) -> CompletedMarker {
  let expr = expr::expr(p);

  let kind = if let Some(op) = assignment_op(p, 0) {
    p.bump(op);
    expr::expr(p);
    AssignStmt
  } else {
    match p.current() {
      // `subject in place;` hands the subject a destination. The grammar takes
      // any subject: which ones can be returned into a place (calls, overloaded
      // operators, builtin arithmetic) is the semantic layer's business.
      T![in] => {
        p.bump(T![in]);
        expr::expr(p);
        PlaceCallStmt
      }
      T![;] => ExprStmt,
      // Not a statement after all; leave the expression to the caller.
      _ => {
        p.abandon(m);
        return expr;
      }
    }
  };

  p.expect(T![;]);
  p.complete(m, kind)
}

pub fn assignment_op(p: &Parser, n: usize) -> Option<SyntaxKind> {
  match p.nth(n) {
    T![=] if !p.nth_at(n, T![==]) => Some(T![=]),
    T![+] if p.nth_at(n, T![+=]) => Some(T![+=]),
    T![-] if p.nth_at(n, T![-=]) => Some(T![-=]),
    T![*] if p.nth_at(n, T![*=]) => Some(T![*=]),
    T![/] if p.nth_at(n, T![/=]) => Some(T![/=]),
    T![%] if p.nth_at(n, T![%=]) => Some(T![%=]),
    T![<] if p.nth_at(n, T![<<=]) => Some(T![<<=]),
    T![>] if p.nth_at(n, T![>>=]) => Some(T![>>=]),
    _ => None,
  }
}

fn let_stmt(p: &mut Parser, m: Marker) -> CompletedMarker {
  p.expect(T![let]);
  pat::pattern(p);
  if p.bump_if(T![:]) {
    types::type_(p);
  }
  if p.bump_if(T![=]) {
    expr::expr(p);
  }
  p.expect(T![;]);

  p.complete(m, LetStmt)
}

fn short_let_stmt(p: &mut Parser, m: Marker) -> CompletedMarker {
  // The dispatch above only enters here on an identifier.
  name(p);
  if !p.bump_if(T![:=]) {
    let kind = p.nth(1);

    if kind == T![=] {
      p.error(Error::Expected {
        expected: T![:=],
        actual: T![:],
      });

      p.bump_any();
      p.bump_any();
    } else {
      p.expect(T![:]);

      p.error(Error::Expected {
        expected: T![:=],
        actual: kind,
      });

      match kind {
        T![mod] | T![fn] | T![type] | T![nominal] => p.bump_any(),
        _ => _ = types::type_(p),
      };

      p.expect(T![=]);
    }
  }
  expr::expr(p);
  p.expect(T![;]);

  p.complete(m, ShortLetStmt)
}

fn using_stmt(p: &mut Parser, m: Marker) -> CompletedMarker {
  p.expect(T![using]);
  items::using_tree(p);
  p.expect(T![;]);

  p.complete(m, UsingStmt)
}
