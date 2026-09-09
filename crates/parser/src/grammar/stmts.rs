use super::prelude::*;

pub fn stmt(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  attrs::attrs(p);

  match p.current() {
    T![let] => let_stmt(p, m),
    Ident if p.nth_at(1, T![:]) && !p.nth_at(1, T![::]) => short_let_stmt(p, m),
    T![using] => using_stmt(p, m),

    _ => {
      let expr = expr::expr(p);
      if let Some(op) = assignment_op(p, 0) {
        p.bump(op);
        expr::expr(p);
        p.expect(T![;]);
        p.complete(m, AssignStmt)
      } else if p.at(T![;]) {
        p.expect(T![;]);
        p.complete(m, ExprStmt)
      } else {
        p.abandon(m);
        expr
      }
    }
  }
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
  p.expect(Ident);
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
