use super::prelude::*;

pub fn expr(p: &mut Parser) -> CompletedMarker {
  expr_bp(p, Bp::Atom)
}

fn expr_bp(p: &mut Parser, min_bp: Bp) -> CompletedMarker {
  let mut lhs = expr_lhs(p);

  loop {
    lhs = match p.current() {
      T!['('] => {
        let m = p.precede(lhs);
        arg_list(p);
        p.complete(m, CallExpr)
      }
      T!['['] => {
        let m = p.precede(lhs);
        index_arg(p);
        p.complete(m, IndexExpr)
      }
      T![+] if p.at(T![++]) => {
        let m = p.precede(lhs);
        p.bump(T![++]);
        p.complete(m, PostfixExpr)
      }
      T![-] if p.at(T![--]) => {
        let m = p.precede(lhs);
        p.bump(T![--]);
        p.complete(m, PostfixExpr)
      }
      T![as] => {
        let m = p.precede(lhs);
        p.bump(T![as]);
        types::type_(p);
        p.complete(m, CastExpr)
      }
      T![.] => {
        let m = p.precede(lhs);
        p.bump(T![.]);

        let kind = if matches!(p.nth(1), T!['('] | T![:]) {
          paths::path(p);
          arg_list(p);
          MethodCallExpr
        } else {
          if !p.bump_if(Int) {
            expect_ident(p);
          }
          FieldExpr
        };

        p.complete(m, kind)
      }
      _ => break,
    };
  }

  while let Some((op, lbp, rbp)) = binary_op(p) {
    // This is a modified Pratt parser.
    //
    // We use `<=` instead of the more common `<` so an operator with the
    // same binding power is not consumed by the recursive RHS parse. For
    // normal binary operators, that keeps equal-precedence operators
    // left-associative.
    //
    // For chain operators, this lets the outer `chain_expr` keep consuming
    // the sequence into one flat `ChainExpr`. If RHS parsing consumed
    // equal-precedence chain operators, chains like `a && b && c` would
    // become nested again. Chain comparison directionality is checked later.
    if lbp <= min_bp {
      break;
    }

    if let Some(group) = chain_group(op) {
      lhs = chain_expr(p, lhs, group, min_bp);
      continue;
    }

    let m = p.precede(lhs);
    p.bump(op);
    expr_bp(p, rbp);
    lhs = p.complete(m, BinaryExpr);
  }

  lhs
}

fn chain_expr(
  p: &mut Parser,
  lhs: CompletedMarker,
  group: ChainGroup,
  min_bp: Bp,
) -> CompletedMarker {
  let m = p.precede(lhs);

  while let Some((op, lbp, rbp)) = binary_op(p) {
    if lbp <= min_bp || chain_group(op) != Some(group) {
      break;
    }

    p.bump(op);
    expr_bp(p, rbp);
  }

  p.complete(m, ChainExpr)
}

fn expr_lhs(p: &mut Parser) -> CompletedMarker {
  match p.current() {
    T![-] | T![!] | T![*] => prefix_expr(p),
    T![&] => ref_expr(p),
    T![return] => return_expr(p),
    T![break] => break_expr(p),
    T![cont] => cont_expr(p),
    _ => expr_atom(p),
  }
}

fn expr_atom(p: &mut Parser) -> CompletedMarker {
  match p.current() {
    T![_] => wildcard_expr(p),
    Int | Char | Byte | String | RawString | T![true] | T![false] => literal_expr(p),
    Ident | T![self] | T![super] | T![unit] => path_expr(p),
    T!['('] => tuple_or_paren_expr(p),
    T!['['] => array_expr(p),
    T!['{'] => brace_expr(p),
    T![fn] => closure_expr(p),
    Label => labeled_expr(p),
    T![case] => case_expr(p),
    T![if] => if_expr(p),
    T![while] => while_expr(p),
    T![for] => for_expr(p),
    T![iterate] => iterate_expr(p),
    _ => error_expr(p),
  }
}

fn wildcard_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![_]);
  p.complete(m, WildcardExpr)
}

fn literal_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  match p.current() {
    String | RawString => {
      while p.at(String) || p.at(RawString) {
        p.bump_any();
      }
    }
    Int | Char | Byte => p.bump_any(),
    T![true] => p.bump(T![true]),
    T![false] => p.bump(T![false]),
    _ => unreachable!(),
  }
  p.complete(m, LiteralExpr)
}

fn path_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  paths::path(p);
  p.complete(m, PathExpr)
}

pub fn tuple_or_paren_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  let mut n_exprs = 0;
  let mut trailing_comma = false;

  p.expect(T!['(']);
  while !p.at_eof() && !p.at(T![')']) {
    if at_expr_end(p) {
      break;
    }
    expr(p);
    n_exprs += 1;
    if p.bump_if(T![,]) {
      trailing_comma = true;
    } else {
      break;
    }
  }
  p.expect(T![')']);

  let kind = if n_exprs == 1 && !trailing_comma {
    ParenExpr
  } else {
    TupleExpr
  };
  p.complete(m, kind)
}

pub fn array_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['[']);
  let kind = 'blk: {
    if p.at(T![']']) {
      break 'blk ArrayExpr;
    }

    expr(p);

    if p.bump_if(T![;]) {
      expr(p);
      break 'blk RepeatExpr;
    }

    if !p.bump_if(T![,]) {
      break 'blk ArrayExpr;
    }

    while !p.at_eof() && !p.at(T![']']) {
      expr(p);
      if !p.bump_if(T![,]) {
        break;
      }
    }
    ArrayExpr
  };
  p.expect(T![']']);

  p.complete(m, kind)
}

pub fn brace_expr(p: &mut Parser) -> CompletedMarker {
  if at_struct_expr(p) {
    struct_expr(p)
  } else {
    block_expr(p)
  }
}

pub fn struct_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['{']);
  while !p.at_eof() && !p.at(T!['}']) {
    field_value(p);
    if p.bump_if(T![,]) {
      continue;
    }
    if p.at(T!['}']) {
      break;
    }
    p.error(Error::Expected {
      expected: T!['}'],
      actual: p.current(),
    });
    recover_balanced(p, at_struct_field_recovery);
  }
  p.expect(T!['}']);

  p.complete(m, StructExpr)
}

fn field_value(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  types::field_name(p);
  if p.bump_if(T![:]) {
    expr(p);
  }
  p.complete(m, FieldValue)
}

fn at_struct_field_recovery(p: &Parser) -> bool {
  p.at(T![,]) || p.at(T!['}'])
}

pub fn block_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['{']);
  while !p.at_eof() && !p.at(T!['}']) {
    let pos = p.pos();
    stmts::stmt(p);
    if p.pos() == pos {
      p.bump_any();
    }
  }
  p.expect(T!['}']);

  p.complete(m, BlockExpr)
}

fn case_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![case]);

  p.expect(T!['{']);
  case_arm_list(p);
  p.expect(T!['}']);

  p.complete(m, CaseExpr)
}

pub fn case_arm_list(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  while !p.at_eof() && !p.at(T!['}']) {
    case_arm(p);
    if !p.bump_if(T![,]) {
      break;
    }
  }
  p.complete(m, CaseArmList)
}

fn case_arm(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  expr(p);
  p.expect(T![=]);
  expr(p);
  p.complete(m, CaseArm)
}

fn if_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![if]);
  expr(p);
  block_expr(p);
  else_clause(p);
  p.complete(m, IfExpr)
}

fn while_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![while]);
  expr(p);
  block_expr(p);
  else_clause(p);
  p.complete(m, WhileExpr)
}

fn for_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![for]);
  pat::pattern(p);
  p.expect(T![in]);
  expr(p);
  block_expr(p);
  else_clause(p);
  p.complete(m, ForExpr)
}

fn iterate_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![iterate]);
  pat::pattern(p);
  p.expect(T![=]);
  expr(p);
  block_expr(p);
  p.complete(m, IterateExpr)
}

fn else_clause(p: &mut Parser) {
  if !p.at(T![else]) {
    return;
  }

  let m = p.start();
  p.bump(T![else]);
  block_expr(p);
  p.complete(m, ElseClause);
}

fn prefix_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump(p.current());
  expr_bp(p, Bp::Prefix);
  p.complete(m, PrefixExpr)
}

fn ref_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump(T![&]);
  p.bump_if(T![mut]);
  expr_bp(p, Bp::Prefix);
  p.complete(m, RefExpr)
}

fn return_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump(T![return]);
  if !at_expr_end(p) {
    expr(p);
  }
  p.complete(m, ReturnExpr)
}

fn break_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump(T![break]);
  p.bump_if(Label);
  if !at_expr_end(p) {
    expr(p);
  }
  p.complete(m, BreakExpr)
}

fn cont_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump(T![cont]);
  p.bump_if(Label);
  if !at_expr_end(p) {
    expr(p);
  }
  p.complete(m, ContinueExpr)
}

fn closure_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump(T![fn]);
  if p.at(T!['(']) {
    func::parameter_list(p);
  }
  if p.bump_if(T![->]) {
    types::type_(p);
  }
  p.expect(T![=]);
  expr(p);
  p.complete(m, ClosureExpr)
}

fn labeled_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.bump(Label);
  p.expect(T![:]);
  match p.current() {
    T!['{'] => block_expr(p),
    T![while] => while_expr(p),
    T![for] => for_expr(p),
    T![iterate] => iterate_expr(p),
    _ => error_expr(p),
  };
  p.complete(m, LabeledExpr)
}

pub fn arg_list(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['(']);
  while !p.at_eof() && !p.at(T![')']) {
    expr(p);
    if !p.bump_if(T![,]) {
      break;
    }
  }
  p.expect(T![')']);

  p.complete(m, ArgList)
}

pub fn index_arg(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T!['[']);
  expr(p);
  p.expect(T![']']);
  p.complete(m, IndexArg)
}

fn error_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.error(Error::Expected {
    expected: Ident,
    actual: p.current(),
  });
  recover_balanced(p, at_expr_end);
  p.complete(m, ErrorExpr)
}

fn at_struct_expr(p: &Parser) -> bool {
  // 0: {
  assert_eq!(p.current(), T!['{']);

  // 1: }
  if p.nth_at(1, T!['}']) {
    return true;
  }

  // 1: Ident | _
  if !matches!(p.nth(1), Ident | T![_]) {
    return false;
  }

  // 2: , | }
  if matches!(p.nth(2), T![,] | T!['}']) {
    return true;
  }

  // 2: :
  if !nth_at_single_colon(p, 2) {
    return false;
  }

  // 3: kind | =
  if matches!(p.nth(3), T![=] | T![mod] | T![fn] | T![type] | T![nominal]) {
    return false;
  }

  let mut idx = 3;
  let mut expected_closes = Vec::new();
  loop {
    if stmts::assignment_op(p, idx).is_some() {
      return false;
    }

    match p.nth(idx) {
      Eof => return false,
      T![,] if expected_closes.is_empty() => return true,
      T!['}'] if expected_closes.is_empty() => return true,
      T![;] if expected_closes.is_empty() => return false,
      T!['('] => expected_closes.push(T![')']),
      T!['['] => expected_closes.push(T![']']),
      T!['{'] => expected_closes.push(T!['}']),
      d @ (T![')'] | T![']'] | T!['}']) if expected_closes.pop() != Some(d) => {
        return false;
      }
      _ => (),
    }

    idx += 1;
  }
}

fn binary_op(p: &Parser) -> Option<(SyntaxKind, Bp, Bp)> {
  if stmts::assignment_op(p, 0).is_some() {
    return None;
  }

  let op = match p.current() {
    T![|] if p.at(T![||]) => T![||],
    T![&] if p.at(T![&&]) => T![&&],
    T![=] if p.at(T![==]) => T![==],
    T![!] if p.at(T![!=]) => T![!=],
    T![<] if p.at(T![<=]) => T![<=],
    T![>] if p.at(T![>=]) => T![>=],
    T![<] if p.at(T![<<]) => T![<<],
    T![>] if p.at(T![>>]) => T![>>],
    T![~] if p.at(T![~=]) => T![~=],
    op @ (T![<] | T![>] | T![+] | T![-] | T![*] | T![/] | T![%] | T![~]) => op,
    _ => return None,
  };

  let p2 = |x| (x, x);
  let bp = match op {
    T![||] | T![&&] => p2(Bp::Logical),
    T![==] | T![!=] | T![<] | T![>] | T![<=] | T![>=] => p2(Bp::Cmp),
    T![<<] | T![>>] => (Bp::ShiftL, Bp::ShiftR),
    T![+] | T![-] => (Bp::AddL, Bp::AddR),
    T![*] | T![/] | T![%] => (Bp::MulL, Bp::MulR),
    T![~] | T![~=] => p2(Bp::Range),
    _ => unreachable!(),
  };

  Some((op, bp.0, bp.1))
}

fn chain_group(op: SyntaxKind) -> Option<ChainGroup> {
  match op {
    T![||] | T![&&] => Some(ChainGroup::Logical),
    T![==] | T![!=] | T![<] | T![>] | T![<=] | T![>=] => Some(ChainGroup::Comparison),
    _ => None,
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChainGroup {
  Logical,
  Comparison,
}

fn at_expr_end(p: &Parser) -> bool {
  p.at_eof() || p.at(T![;]) || p.at(T![,]) || p.at(T![')']) || p.at(T![']']) || p.at(T!['}'])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bp {
  Atom,

  Range,

  Logical,

  Cmp,

  ShiftL,
  ShiftR,

  AddL,
  AddR,

  MulL,
  MulR,

  Prefix,
}
