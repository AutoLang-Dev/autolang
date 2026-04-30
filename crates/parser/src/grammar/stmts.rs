use super::prelude::*;

pub fn stmt(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  attrs::attrs(p);

  if is_item_start(p) {
    return items::item(p, Some(m));
  }

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
