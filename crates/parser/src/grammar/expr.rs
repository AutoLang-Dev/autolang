use super::prelude::*;

/// Whether `expr { .. }` may be parsed as a call taking one record argument.
///
/// Constructs that own a trailing block (`if`, `while`, `for`) parse their
/// whole head with [`BraceCall::Forbid`] so that the block stays theirs:
/// `if a == b { c }` must not parse `b { c }` as a call. The flag therefore has
/// to travel through *every* operator in the head — one `Allow` leaking into a
/// nested level is enough to swallow the block.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BraceCall {
  Allow,
  Forbid,
}

pub fn expr(p: &mut Parser) -> CompletedMarker {
  expr_bp(p, Bp::Atom, BraceCall::Allow)
}

fn expr_no_brace_call(p: &mut Parser) -> CompletedMarker {
  expr_bp(p, Bp::Atom, BraceCall::Forbid)
}

fn expr_bp(p: &mut Parser, min_bp: Bp, brace_call: BraceCall) -> CompletedMarker {
  let mut lhs = expr_lhs(p, brace_call);

  loop {
    lhs = match p.current() {
      k @ (T!['('] | T!['{']) if k != T!['{'] || brace_call == BraceCall::Allow => {
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

        let kind = if matches!(p.nth(1), T!['('] | T!['{'] | T![:]) {
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
      lhs = chain_expr(p, lhs, group, min_bp, brace_call);
      continue;
    }

    let m = p.precede(lhs);
    p.bump(op);
    expr_bp(p, rbp, brace_call);
    lhs = p.complete(m, BinaryExpr);
  }

  lhs
}

fn chain_expr(
  p: &mut Parser,
  lhs: CompletedMarker,
  group: ChainGroup,
  min_bp: Bp,
  brace_call: BraceCall,
) -> CompletedMarker {
  let m = p.precede(lhs);

  while let Some((op, lbp, rbp)) = binary_op(p) {
    if lbp <= min_bp || chain_group(op) != Some(group) {
      break;
    }

    p.bump(op);
    expr_bp(p, rbp, brace_call);
  }

  p.complete(m, ChainExpr)
}

fn expr_lhs(p: &mut Parser, brace_call: BraceCall) -> CompletedMarker {
  match p.current() {
    T![-] | T![!] | T![*] => prefix_expr(p, brace_call),
    T![&] => ref_expr(p, brace_call),
    T![return] => return_expr(p),
    T![break] => break_expr(p),
    T![cont] => cont_expr(p),
    T![_] => wildcard_expr(p),
    Int | Char | Byte | String | RawString | T![true] | T![false] => literal_expr(p),
    Ident | T![self] | T![super] | T![unit] => path_expr(p),
    T!['('] => paren_expr(p, false),
    T!['['] => array_expr(p),
    T!['{'] => brace_expr(p),
    T!['\\'] => closure_expr(p),
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

pub fn paren_expr(p: &mut Parser, mut tuple: bool) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['(']);
  tuple = tuple || (p.at(Ident) && p.nth_at(1, T![:])) || p.at(T![')']);

  if !tuple {
    let first = expr(p);
    if p.at(T![,]) {
      tuple = true;
      let m = p.precede(first);
      p.complete(m, ExprField);
      p.bump_any();
    }
  }

  while !p.at_eof() && !p.at(T![')']) {
    if at_expr_end(p) {
      break;
    }
    expr_field(p, false);
    if !p.bump_if(T![,]) {
      break;
    }
  }
  p.expect(T![')']);

  let kind = if tuple { TupleExpr } else { ParenExpr };
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
  if at_record_expr(p) {
    record_expr(p)
  } else {
    block_expr(p)
  }
}

pub fn record_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();

  p.expect(T!['{']);
  while !p.at_eof() && !p.at(T!['}']) {
    expr_field(p, true);
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
    recover_balanced(p, at_record_field_recovery);
  }
  p.expect(T!['}']);

  p.complete(m, RecordExpr)
}

fn expr_field(p: &mut Parser, record: bool) -> CompletedMarker {
  let m = p.start();
  if record {
    types::field_name(p);
    if p.bump_if(T![:]) {
      expr(p);
    }
  } else {
    if p.at(Ident) && p.nth_at(1, T![:]) {
      types::field_name(p);
      p.expect(T![:]);
    }
    expr(p);
  }
  p.complete(m, ExprField)
}

fn at_record_field_recovery(p: &Parser) -> bool {
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
  expr_no_brace_call(p);
  block_expr(p);
  else_clause(p);
  p.complete(m, IfExpr)
}

fn while_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![while]);
  expr_no_brace_call(p);
  block_expr(p);
  else_clause(p);
  p.complete(m, WhileExpr)
}

fn for_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![for]);
  pat::pattern(p);
  p.expect(T![in]);
  expr_no_brace_call(p);
  block_expr(p);
  else_clause(p);
  p.complete(m, ForExpr)
}

fn iterate_expr(p: &mut Parser) -> CompletedMarker {
  let m = p.start();
  p.expect(T![iterate]);
  pat::pattern(p);
  p.expect(T![=]);
  expr_no_brace_call(p);
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

fn prefix_expr(p: &mut Parser, brace_call: BraceCall) -> CompletedMarker {
  let m = p.start();
  p.bump(p.current());
  expr_bp(p, Bp::Prefix, brace_call);
  p.complete(m, PrefixExpr)
}

fn ref_expr(p: &mut Parser, brace_call: BraceCall) -> CompletedMarker {
  let m = p.start();
  p.bump(T![&]);
  p.bump_if(T![mut]);
  expr_bp(p, Bp::Prefix, brace_call);
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
  p.bump(T!['\\']);
  if !p.at(T![.]) {
    pat::pattern(p);
  }
  p.bump_if(T![.]);
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

  match p.current() {
    T!['('] => _ = paren_expr(p, true),
    T!['{'] => _ = record_expr(p),
    kind => {
      p.error(Error::Expected {
        expected: T!['('],
        actual: kind,
      });
    }
  };

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

fn at_record_expr(p: &Parser) -> bool {
  // 0: {
  assert_eq!(p.current(), T!['{']);

  // 1: }
  if p.nth_at(1, T!['}']) {
    return true;
  }

  // 1: Ident
  if !matches!(p.nth(1), Ident) {
    return false;
  }

  // 2: := | ::
  if p.nth_at(2, T![:=]) || p.nth_at(2, T![::]) {
    return false;
  }

  // 2: : | ,
  matches!(p.nth(2), T![:] | T![,] | T!['}'])
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
