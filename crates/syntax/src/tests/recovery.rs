//! Malformed and truncated trees stay quiet instead of panicking.

use super::probe::probe_red;
use crate::Red;
use crate::SyntaxKind::*;
use crate::ast;
use crate::frag;

#[test]
fn malformed_trees_stay_quiet() {
  macro_rules! probe {
    ($kind:ident, $ty:ident, $($f:ident),* $(,)?) => {
      if let Some(node) = <ast::$ty>::new(probe_red($kind)) {
        $( let _ = node.$f(); )*
      }
    };
  }

  probe!(ArgList, ArgList, args);
  probe!(ArrayExpr, ArrayExpr, values);
  probe!(ArrayType, ArrayType, len, ty);
  probe!(AssignStmt, AssignStmt, attr, lhs, op, rhs);
  probe!(Attr, Attr, items);
  probe!(AttrItem, AttrItem, arg, attr);
  probe!(BlockExpr, BlockExpr, expr, stmts);
  probe!(BreakExpr, BreakExpr, expr, label);
  probe!(CallExpr, CallExpr, args, callee);
  probe!(CastExpr, CastExpr, expr, ty);
  probe!(ContinueExpr, ContinueExpr, expr, label);
  probe!(EmptyItem, EmptyItem, attr, vis);
  probe!(ErrorItem, ErrorItem, attr, vis);
  probe!(ExprField, ExprField, name, value);
  probe!(ExprStmt, ExprStmt, attr, expr);
  probe!(FieldExpr, FieldExpr, expr, field);
  probe!(FnType, FnType, mutable, params, ret);
  probe!(ForExpr, ForExpr, else_branch, pat, range, then);
  probe!(FunctionItem, FunctionItem, attr, body, name, ty, vis);
  probe!(IdentPat, IdentPat, mutable, name);
  probe!(IfExpr, IfExpr, cond, else_branch, then);
  probe!(IndexArg, IndexArg, index);
  probe!(IndexExpr, IndexExpr, expr, index);
  probe!(IterateExpr, IterateExpr, body, init, pat);
  probe!(LabeledExpr, LabeledExpr, expr, label);
  probe!(LetStmt, LetStmt, attr, init, pat, ty);
  probe!(ModuleItem, ModuleItem, attr, items, name, vis);
  probe!(ParenExpr, ParenExpr, expr);
  probe!(PatField, PatField, name, pat);
  probe!(PathExpr, PathExpr, path);
  probe!(PathType, PathType, path);
  probe!(PlaceCallStmt, PlaceCallStmt, attr, place, subject);
  probe!(PtrType, PtrType, mutable, pointee);
  probe!(RecordExpr, RecordExpr, fields);
  probe!(RecordPat, RecordPat, fields);
  probe!(RecordType, RecordType, fields);
  probe!(RefExpr, RefExpr, expr, mutable);
  probe!(RefType, RefType, mutable, pointee);
  probe!(RepeatExpr, RepeatExpr, len, value);
  probe!(ReturnExpr, ReturnExpr, expr);
  probe!(SourceFile, Root, items);
  probe!(ShortLetStmt, ShortLetStmt, attr, init, name);
  probe!(SliceType, SliceType, ty);
  probe!(TupleExpr, TupleExpr, fields);
  probe!(TuplePat, TuplePat, fields);
  probe!(TupleType, TupleType, fields);
  probe!(TypeField, TypeField, attr, name, ty, vis);
  probe!(TypeItem, TypeItem, attr, kind, name, ty, vis);
  probe!(UsingItem, UsingItem, attr, vis);
  probe!(UsingStmt, UsingStmt, attr);
  probe!(WhileExpr, WhileExpr, cond, else_branch, then);

  let ty = ast::FnType::new(probe_red(FnType)).expect("FnType");
  assert!(ty.params().is_none());
  assert!(ty.ret().is_none());
  assert!(!ty.mutable());

  let (green, _) = frag! {
    BlockExpr { OpenBrace ExprStmt { LiteralExpr { Int("1") } Semi } CloseBrace }
  }
  .finish();
  let block = ast::BlockExpr::new(Red::new_root(green)).expect("BlockExpr");
  assert_eq!(block.stmts().len(), 1);
  assert!(block.expr().is_none());
}
