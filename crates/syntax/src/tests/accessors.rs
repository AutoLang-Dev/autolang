//! The accessors each node kind exposes.

use super::probe::probe_red;
use crate::Red;
use crate::SyntaxKind::*;
use crate::ast;
use crate::ast::Node;
use crate::frag;

#[test]
fn tuple_and_record_patterns_are_reachable() {
  let tuple = frag! {
    LetStmt {
      KwLet
      TuplePat {
        OpenParen
        PatField { IdentPat { Name { Ident("a") } } }
        Comma
        PatField { IdentPat { Name { Ident("b") } } }
        CloseParen
      }
      Semi
    }
  }
  .red();
  let let_stmt = ast::LetStmt::new(tuple).expect("LetStmt");

  assert!(
    matches!(let_stmt.pat(), Some(ast::Pattern::Tuple(_))),
    "`let (a, b)` must model its tuple pattern"
  );

  let record = frag! {
    RecordPat { OpenBrace PatField { Name { Ident("a") } } CloseBrace }
  }
  .red();

  assert!(matches!(
    ast::Pattern::new(record),
    Some(ast::Pattern::Record(_))
  ));
}

#[test]
fn path_segments_carry_their_names() {
  let (green, src) = frag! {
    PathExpr {
      Path {
        PathSegment { Name { Ident("std") } }
        ColonColon
        PathSegment { Name { Ident("io") } }
      }
    }
  }
  .finish();
  let expr = ast::PathExpr::new(Red::new_root(green)).expect("PathExpr");
  let path = expr.path().expect("path");

  let names: Vec<&str> = path
    .segments()
    .iter()
    .map(|segment| match segment.kind() {
      ast::PathSegmentKind::Name(name) => &src[name.red().range()],
      other => panic!("expected a name segment, got {other:?}"),
    })
    .collect();

  assert_eq!(names, ["std", "io"]);
}

#[test]
fn keyword_path_segments_are_not_names() {
  let (green, src) = frag! {
    PathExpr {
      Path {
        PathSegment { KwSelf }
        ColonColon
        PathSegment { KwSuper }
        ColonColon
        PathSegment { KwUnit }
        ColonColon
        PathSegment { Name { Ident("x") } }
      }
    }
  }
  .finish();
  let expr = ast::PathExpr::new(Red::new_root(green)).expect("PathExpr");
  let path = expr.path().expect("path");
  let segments = path.segments();
  let kinds: Vec<ast::PathSegmentKind> = segments.iter().map(|segment| segment.kind()).collect();

  assert_eq!(
    &kinds[..3],
    [
      ast::PathSegmentKind::Self_,
      ast::PathSegmentKind::Super,
      ast::PathSegmentKind::Unit,
    ]
  );
  assert!(matches!(kinds[3], ast::PathSegmentKind::Name(_)));
  assert_eq!(&src[segments[0].red().range()], "self");
  assert_eq!(&src[segments[3].red().range()], "x");

  let empty = ast::PathSegment::new(frag! { PathSegment {} }.red()).expect("PathSegment");
  assert_eq!(empty.kind(), ast::PathSegmentKind::Error);
}

#[test]
fn chain_prefix_and_postfix_expose_their_parts() {
  let (green, src) = frag! {
    ChainExpr {
      PathExpr { Path { PathSegment { Name { Ident("a") } } } }
      EqEq
      PathExpr { Path { PathSegment { Name { Ident("b") } } } }
      Lt
      PathExpr { Path { PathSegment { Name { Ident("c") } } } }
    }
  }
  .finish();
  let chain = ast::ChainExpr::new(Red::new_root(green)).expect("ChainExpr");
  let operands: Vec<&str> = chain
    .operands()
    .iter()
    .map(|expr| &src[expr.red().range()])
    .collect();
  let ops: Vec<_> = chain.op_tokens().iter().map(|token| token.kind()).collect();

  assert_eq!(operands, ["a", "b", "c"]);
  assert_eq!(ops, [EqEq, Lt]);

  let (green, src) = frag! {
    PrefixExpr { Minus PathExpr { Path { PathSegment { Name { Ident("a") } } } } }
  }
  .finish();
  let prefix = ast::PrefixExpr::new(Red::new_root(green)).expect("PrefixExpr");

  assert_eq!(prefix.op_token().expect("op").kind(), Minus);
  assert_eq!(&src[prefix.expr().expect("expr").red().range()], "a");

  let (green, src) = frag! {
    PostfixExpr { PathExpr { Path { PathSegment { Name { Ident("p") } } } } Dot Star }
  }
  .finish();
  let postfix = ast::PostfixExpr::new(Red::new_root(green)).expect("PostfixExpr");

  assert_eq!(postfix.op_token().expect("op").kind(), Star);
  assert_eq!(&src[postfix.expr().expect("expr").red().range()], "p");
}

#[test]
fn literal_text_covers_every_token() {
  let (green, src) = frag! {
    LiteralExpr { String("\"a\"") Whitespace(" ") String("\"b\"") }
  }
  .finish();
  let literal = ast::LiteralExpr::new(Red::new_root(green)).expect("LiteralExpr");

  assert_eq!(&src[literal.red().range()], "\"a\" \"b\"");
}

#[test]
fn binary_and_method_call_expose_their_parts() {
  let (green, src) = frag! {
    BinaryExpr {
      PathExpr { Path { PathSegment { Name { Ident("a") } } } }
      Plus
      LiteralExpr { Int("1") }
    }
  }
  .finish();
  let binary = ast::BinaryExpr::new(Red::new_root(green)).expect("BinaryExpr");

  assert_eq!(&src[binary.lhs().expect("lhs").red().range()], "a");
  assert_eq!(binary.op_token().expect("op").kind(), Plus);
  assert_eq!(&src[binary.rhs().expect("rhs").red().range()], "1");

  let (green, src) = frag! {
    MethodCallExpr {
      PathExpr { Path { PathSegment { Name { Ident("a") } } } }
      Dot
      Name { Ident("b") }
      ArgList { TupleExpr { OpenParen ExprField { LiteralExpr { Int("1") } } CloseParen } }
    }
  }
  .finish();
  let call = ast::MethodCallExpr::new(Red::new_root(green)).expect("MethodCallExpr");
  let args = call.args().expect("args");

  assert_eq!(&src[call.receiver().expect("receiver").red().range()], "a");
  assert_eq!(&src[call.name().expect("name").red().range()], "b");
  assert_eq!(
    &src[args.fields().first().expect("field").red().range()],
    "1"
  );
}

#[test]
fn closure_and_else_expose_their_parts() {
  let (green, src) = frag! {
    ClosureExpr {
      IdentPat { Name { Ident("x") } }
      Dot
      PathExpr { Path { PathSegment { Name { Ident("x") } } } }
    }
  }
  .finish();
  let closure = ast::ClosureExpr::new(Red::new_root(green)).expect("ClosureExpr");

  assert!(closure.pat().is_some());
  assert_eq!(&src[closure.body().expect("body").red().range()], "x");

  let (green, src) = frag! {
    ElseClause { KwElse BlockExpr { OpenBrace LiteralExpr { Int("2") } CloseBrace } }
  }
  .finish();
  let else_clause = ast::ElseClause::new(Red::new_root(green)).expect("ElseClause");

  assert_eq!(
    &src[else_clause.block().expect("block").red().range()],
    "{2}"
  );
}

#[test]
fn attrs_returns_every_attribute() {
  let (green, src) = frag! {
    ModuleItem {
      Attr { Hash AttrInner { OpenBrack AttrItem { Path { PathSegment { Name { Ident("a") } } } } CloseBrack } }
      Attr { Hash AttrInner { OpenBrack AttrItem { Path { PathSegment { Name { Ident("b") } } } } CloseBrack } }
      Name { Ident("m") }
      Colon
      KwMod
      Semi
    }
  }
  .finish();
  let item = ast::ModuleItem::new(Red::new_root(green)).expect("ModuleItem");

  let attrs: Vec<&str> = item
    .attrs()
    .iter()
    .map(|attr| &src[attr.red().range()])
    .collect();

  assert_eq!(attrs, ["#[a]", "#[b]"]);
  assert_eq!(&src[item.attrs()[0].red().range()], "#[a]");
}

#[test]
fn using_trees_come_in_three_shapes() {
  let (green, src) = frag! {
    UsingItem {
      KwUsing
      UsingUnitName { Name { Ident("foo") } }
      ColonColon
      UsingTree {
        UsingTreeList {
          OpenBrace
          UsingTree { Name { Ident("bar") } }
          Comma
          UsingTree {
            Name { Ident("baz") }
            Rename { KwAs Name { Ident("qux") } }
          }
          CloseBrace
        }
      }
      Semi
    }
  }
  .finish();
  let item = ast::UsingItem::new(Red::new_root(green)).expect("UsingItem");

  let ast::UsingTreeKind::List { list } = item.tree().expect("tree").kind().unwrap() else {
    panic!("expected a list below a path");
  };

  let trees = list.trees();
  assert_eq!(trees.len(), 2);
  assert!(matches!(
    trees[0].kind().unwrap(),
    ast::UsingTreeKind::Leaf { rename: None, .. }
  ));

  let ast::UsingTreeKind::Leaf { name, rename } = trees[1].kind().unwrap() else {
    panic!("expected a leaf");
  };
  assert_eq!(&src[name.red().range()], "baz");
  let target = rename.expect("a rename").name().expect("a target");
  assert_eq!(&src[target.red().range()], "qux");
}

#[test]
fn a_using_tree_is_a_glob_when_it_has_one() {
  let kind = |red: Red| ast::UsingTree::new(red).expect("UsingTree").kind();

  // `a::_` imports every name of `a`; a bare `_` does the same for the tree it
  // sits in.
  let outer = ast::UsingTree::new(
    frag! { UsingTree { Name { Ident("a") } ColonColon UsingTree { Underscore } } }.red(),
  )
  .expect("UsingTree");
  let ast::UsingTreeKind::Path { tree, .. } = outer.kind().unwrap() else {
    panic!("expected a path tree");
  };
  assert!(matches!(tree.kind(), Some(ast::UsingTreeKind::Glob)));
  assert!(matches!(
    kind(frag! { UsingTree { Underscore } }.red()),
    Some(ast::UsingTreeKind::Glob)
  ));

  assert!(kind(probe_red(UsingTree)).is_none());
}

#[test]
fn attribute_arguments_expose_both_shapes() {
  let tree_item = frag! {
    AttrItem {
      Path { PathSegment { Name { Ident("cfg") } } }
      AttrArg {
        DelimitedTokenTree {
          OpenParen
          TokenTree { Int("1") }
          TokenTree { Comma }
          TokenTree { Ident("a") }
          CloseParen
        }
      }
    }
  }
  .red();
  let item = ast::AttrItem::new(tree_item).expect("AttrItem");
  let ast::AttrArg::Tree(tree) = item.arg().expect("arg") else {
    panic!("expected a token tree argument");
  };

  assert_eq!(tree.delimiter(), Some(OpenParen));

  let inner = tree.trees();
  assert_eq!(inner.len(), 3);
  assert_eq!(
    ast::TokenTree::new(inner[1].clone())
      .expect("leaf")
      .token()
      .expect("token")
      .kind(),
    Comma
  );

  let expr_item = frag! {
    AttrItem {
      Path { PathSegment { Name { Ident("a") } } }
      AttrArg { Eq LiteralExpr { Int("1") } }
    }
  }
  .red();
  let item = ast::AttrItem::new(expr_item).expect("AttrItem");

  assert!(matches!(item.arg(), Some(ast::AttrArg::Expr(_))));
}

#[test]
fn ast_nodes_clone_compare_and_hash() {
  let red = frag! { Name { Ident("x") } }.red();
  let name = ast::Name::new(red.clone()).expect("Name");
  let same = ast::Name::new(red).expect("Name");
  let other = ast::Name::new(frag! { Name { Ident("y") } }.red()).expect("Name");

  assert_eq!(name.clone(), name);
  assert_eq!(name, same);
  assert_ne!(name, other);
  assert_eq!(format!("{name:?}"), "Name@0..1");

  let mut set = std::collections::HashSet::new();
  set.insert(name.clone());
  assert!(set.contains(&same));
  assert!(!set.contains(&other));

  let expr = ast::Expr::new(frag! { LiteralExpr { Int("1") } }.red()).expect("Expr");
  assert_eq!(expr.clone(), expr);
  assert!(format!("{expr:?}").starts_with("Literal("));
}

#[test]
fn statements_expose_their_operands() {
  let (green, src) = frag! {
    AssignStmt {
      PathExpr { Path { PathSegment { Name { Ident("a") } } } }
      PlusEq
      LiteralExpr { Int("1") }
      Semi
    }
  }
  .finish();
  let stmt = ast::AssignStmt::new(Red::new_root(green)).expect("AssignStmt");

  assert_eq!(&src[stmt.lhs().expect("lhs").red().range()], "a");
  assert_eq!(stmt.op().expect("op").red().kind(), PlusEq);
  assert_eq!(&src[stmt.rhs().expect("rhs").red().range()], "1");

  let (green, src) = frag! {
    PlaceCallStmt {
      CallExpr {
        PathExpr { Path { PathSegment { Name { Ident("f") } } } }
        ArgList { TupleExpr { OpenParen CloseParen } }
      }
      KwIn
      PathExpr { Path { PathSegment { Name { Ident("dst") } } } }
      Semi
    }
  }
  .finish();
  let stmt = ast::PlaceCallStmt::new(Red::new_root(green)).expect("PlaceCallStmt");

  assert_eq!(&src[stmt.subject().expect("subject").red().range()], "f()");
  assert_eq!(&src[stmt.place().expect("place").red().range()], "dst");
}

#[test]
fn field_index_and_array_accessors_pick_their_children() {
  let (green, src) = frag! {
    FieldExpr {
      PathExpr { Path { PathSegment { Name { Ident("a") } } } }
      Dot
      Name { Ident("b") }
    }
  }
  .finish();
  let field = ast::FieldExpr::new(Red::new_root(green)).expect("FieldExpr");

  assert_eq!(&src[field.expr().expect("expr").red().range()], "a");
  assert_eq!(&src[field.field().expect("field").red().range()], "b");

  let (green, src) = frag! {
    IndexExpr {
      PathExpr { Path { PathSegment { Name { Ident("a") } } } }
      IndexArg { OpenBrack LiteralExpr { Int("0") } CloseBrack }
    }
  }
  .finish();
  let index = ast::IndexExpr::new(Red::new_root(green)).expect("IndexExpr");
  let arg = index.index().expect("index");

  assert_eq!(&src[index.expr().expect("expr").red().range()], "a");
  assert_eq!(&src[arg.index().expect("index").red().range()], "0");

  let (green, src) = frag! {
    ArrayExpr {
      OpenBrack
      LiteralExpr { Int("1") }
      Comma
      LiteralExpr { Int("2") }
      CloseBrack
    }
  }
  .finish();
  let array = ast::ArrayExpr::new(Red::new_root(green)).expect("ArrayExpr");
  let values: Vec<&str> = array
    .values()
    .iter()
    .map(|value| &src[value.red().range()])
    .collect();

  assert_eq!(values, ["1", "2"]);
}

#[test]
fn call_type_and_attribute_accessors_pick_their_children() {
  let (green, src) = frag! {
    CallExpr {
      PathExpr { Path { PathSegment { Name { Ident("f") } } } }
      ArgList { TupleExpr { OpenParen ExprField { LiteralExpr { Int("1") } } CloseParen } }
    }
  }
  .finish();
  let call = ast::CallExpr::new(Red::new_root(green)).expect("CallExpr");
  let args = call.args().expect("args");

  assert_eq!(&src[call.callee().expect("callee").red().range()], "f");
  assert_eq!(&src[args.fields()[0].red().range()], "1");

  let (green, src) = frag! {
    TypeItem {
      Name { Ident("t") }
      Colon
      TypeKind { KwType }
      Eq
      PathType { Path { PathSegment { Name { Ident("int") } } } }
      Semi
    }
  }
  .finish();
  let item = ast::TypeItem::new(Red::new_root(green)).expect("TypeItem");

  assert_eq!(&src[item.name().expect("name").red().range()], "t");
  assert!(matches!(item.kind(), Some(ast::TypeKind::Alias(_))));
  assert_eq!(&src[item.ty().expect("ty").red().range()], "int");

  let (green, src) = frag! {
    Attr {
      Hash
      AttrInner {
        OpenBrack
        AttrItem { Path { PathSegment { Name { Ident("a") } } } }
        CloseBrack
      }
    }
  }
  .finish();
  let attr = ast::Attr::new(Red::new_root(green)).expect("Attr");

  assert_eq!(
    &src[attr.items()[0].attr().expect("attr").red().range()],
    "a"
  );
}

#[test]
fn else_clauses_hang_off_the_condition_or_loop() {
  let (green, src) = frag! {
    IfExpr {
      KwIf
      LiteralExpr { KwTrue }
      BlockExpr { OpenBrace LiteralExpr { Int("1") } CloseBrace }
      ElseClause { KwElse BlockExpr { OpenBrace LiteralExpr { Int("2") } CloseBrace } }
    }
  }
  .finish();
  let if_expr = ast::IfExpr::new(Red::new_root(green)).expect("IfExpr");

  assert_eq!(&src[if_expr.cond().expect("cond").red().range()], "true");
  assert_eq!(&src[if_expr.then().expect("then").red().range()], "{1}");
  assert_eq!(
    &src[if_expr.else_branch().expect("else").red().range()],
    "else{2}"
  );

  let (green, src) = frag! {
    ForExpr {
      KwFor
      IdentPat { Name { Ident("x") } }
      KwIn
      PathExpr { Path { PathSegment { Name { Ident("xs") } } } }
      BlockExpr { OpenBrace LiteralExpr { Int("1") } CloseBrace }
      ElseClause { KwElse BlockExpr { OpenBrace LiteralExpr { Int("2") } CloseBrace } }
    }
  }
  .finish();
  let for_expr = ast::ForExpr::new(Red::new_root(green)).expect("ForExpr");

  assert_eq!(&src[for_expr.pat().expect("pat").red().range()], "x");
  assert_eq!(&src[for_expr.range().expect("range").red().range()], "xs");
  assert_eq!(&src[for_expr.then().expect("then").red().range()], "{1}");
  assert_eq!(
    &src[for_expr
      .else_branch()
      .expect("else")
      .block()
      .expect("block")
      .red()
      .range()],
    "{2}"
  );
}

#[test]
fn mutable_shows_up_in_every_shape() {
  let (green, src) = frag! { IdentPat { KwMut Name { Ident("x") } } }.finish();
  let pat = ast::IdentPat::new(Red::new_root(green)).expect("IdentPat");

  assert!(pat.mutable());
  assert_eq!(&src[pat.name().expect("name").red().range()], "x");

  let plain = frag! { IdentPat { Name { Ident("x") } } }.red();
  assert!(!ast::IdentPat::new(plain).expect("IdentPat").mutable());

  let mutable_ref = frag! {
    RefExpr { Amp KwMut PathExpr { Path { PathSegment { Name { Ident("x") } } } } }
  }
  .red();
  let plain_ref = frag! {
    RefExpr { Amp PathExpr { Path { PathSegment { Name { Ident("x") } } } } }
  }
  .red();
  assert!(ast::RefExpr::new(mutable_ref).expect("RefExpr").mutable());
  assert!(!ast::RefExpr::new(plain_ref).expect("RefExpr").mutable());

  let mutable_ref_ty = frag! {
    RefType { Amp KwMut PathType { Path { PathSegment { Name { Ident("int") } } } } }
  }
  .red();
  assert!(
    ast::RefType::new(mutable_ref_ty)
      .expect("RefType")
      .mutable()
  );

  let (green, ptr_src) = frag! {
    PtrType { Star KwMut PathType { Path { PathSegment { Name { Ident("int") } } } } }
  }
  .finish();
  let ptr = ast::PtrType::new(Red::new_root(green)).expect("PtrType");

  assert!(ptr.mutable());
  assert_eq!(
    &ptr_src[ptr.pointee().expect("pointee").red().range()],
    "int"
  );

  let (green, src) = frag! {
    FnType {
      TupleType { OpenParen CloseParen }
      KwMut
      ThinArrow
      PathType { Path { PathSegment { Name { Ident("int") } } } }
    }
  }
  .finish();
  let ty = ast::FnType::new(Red::new_root(green)).expect("FnType");

  assert!(ty.mutable());
  assert!(ty.params().is_some());
  assert_eq!(&src[ty.ret().expect("ret").red().range()], "int");
}

#[test]
fn modules_expose_their_inline_items() {
  let (green, src) = frag! {
    SourceFile {
      ModuleInner {
        ModuleItem {
          Name { Ident("m") }
          Colon
          KwMod
          Eq
          Module {
            OpenBrace
            ModuleInner {
              FunctionItem {
                Name { Ident("f") }
                Colon
                FnType {
                  TupleType { OpenParen CloseParen }
                  ThinArrow
                  TupleType { OpenParen CloseParen }
                }
                Semi
              }
            }
            CloseBrace
          }
          Semi
        }
      }
    }
  }
  .finish();
  let root = ast::Root::new(Red::new_root(green)).expect("Root");
  let items = root.items();
  let ast::Item::Module(module) = &items[0] else {
    panic!("expected a module item, got {:?}", items[0]);
  };

  assert_eq!(&src[module.name().expect("name").red().range()], "m");

  let inner = module.items().expect("inline module body");
  let ast::Item::Function(function) = &inner[0] else {
    panic!("expected a function item, got {:?}", inner[0]);
  };

  assert_eq!(&src[function.name().expect("name").red().range()], "f");

  let declaration = frag! { ModuleItem { Name { Ident("m") } Colon KwMod Semi } }.red();
  let module = ast::ModuleItem::new(declaration).expect("ModuleItem");

  assert!(module.items().is_none(), "a declaration has no body");
}

#[test]
fn shorthand_fields_carry_only_a_name() {
  let (green, src) = frag! {
    RecordExpr {
      OpenBrace
      ExprField { Name { Ident("a") } }
      Comma
      ExprField { Name { Ident("b") } Colon LiteralExpr { Int("1") } }
      CloseBrace
    }
  }
  .finish();
  let record = ast::RecordExpr::new(Red::new_root(green)).expect("RecordExpr");
  let fields = record.fields();

  assert_eq!(&src[fields[0].name().expect("name").red().range()], "a");
  assert!(
    fields[0].value().is_none(),
    "`{{a}}` binds the name itself, so there is no value node"
  );
  assert_eq!(&src[fields[1].value().expect("value").red().range()], "1");

  let (green, src) = frag! {
    RecordPat { OpenBrace PatField { Name { Ident("a") } } CloseBrace }
  }
  .finish();
  let record = ast::RecordPat::new(Red::new_root(green)).expect("RecordPat");
  let fields = record.fields();

  assert_eq!(&src[fields[0].name().expect("name").red().range()], "a");
  assert!(
    fields[0].pat().is_none(),
    "`{{a}}` binds the name itself, so there is no pattern node"
  );
}
