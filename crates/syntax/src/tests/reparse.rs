use super::{dump_parse, reparse_eq_full};
use crate::{Indel, NodeReparse, Reparse, SyntaxKind, TextRange, parse};

fn token_reparse(reparse: &Reparse) -> &crate::TokenReparse {
  let Reparse::Token(reparse) = reparse else {
    panic!("expected token reparse, got {reparse:?}");
  };
  reparse
}

fn node_reparse(reparse: &Reparse) -> &NodeReparse {
  let Reparse::Node(reparse) = reparse else {
    panic!("expected node reparse, got {reparse:?}");
  };
  reparse
}

#[test]
fn token_reparse_matches_full_parse() {
  let reparse = reparse_eq_full("foo: mod;", 1..2, "a");
  let reparse = token_reparse(&reparse);

  assert_eq!(reparse.dirty.old, TextRange::new(0.into(), 3.into()));
  assert_eq!(reparse.dirty.new, TextRange::new(0.into(), 3.into()));
}

#[test]
fn noop_edit_does_not_reparse() {
  let input = "foo: mod;";
  let parse = parse(input);
  let reparse = parse.reparse(
    input,
    &Indel {
      delete: TextRange::new(0.into(), 3.into()),
      insert: "foo".to_string(),
    },
  );

  assert_eq!(reparse, Reparse::Noop);
}

#[test]
fn string_internal_edit_reparses_token() {
  let input = "x: = \"abcd\";";
  let start = input.find('c').unwrap() as u32;
  let reparse = reparse_eq_full(input, start..start + 1, "cd");

  token_reparse(&reparse);
}

#[test]
fn token_reparse_runs_before_delimiter_balance_check() {
  let input = "x: = \"abcd\";";
  let start = input.find('c').unwrap() as u32;
  let reparse = reparse_eq_full(input, start..start + 1, "{");

  token_reparse(&reparse);
}

#[test]
fn quote_boundary_edit_uses_node_reparse() {
  let input = "x: = { \"aaa\" };";
  let parse = parse(input);
  let quote = input.rfind('"').unwrap() as u32;
  let reparse = parse.reparse(
    input,
    &Indel {
      delete: TextRange::new(quote.into(), (quote + 1).into()),
      insert: String::new(),
    },
  );

  node_reparse(&reparse);
}

#[test]
fn token_edge_next_to_trivia_still_reparses_token() {
  let input = "foo : mod;";
  let end = input.find(' ').unwrap() as u32;
  let reparse = reparse_eq_full(input, end..end, "x");

  token_reparse(&reparse);
}

#[test]
fn token_edge_next_to_non_trivia_uses_node_reparse() {
  let input = "foo: mod;";
  let end = input.find(':').unwrap() as u32;
  let reparse = reparse_eq_full(input, end..end, "x");
  let reparse = node_reparse(&reparse);

  assert_eq!(reparse.reparser, parser::Reparser::ModuleInner);
}

#[test]
fn top_level_item_edit_reparses_module_inner() {
  let input = "x: Int; y: Int;";
  let start = input.find("Int;").unwrap() as u32;

  let reparse = reparse_eq_full(input, start..start + 4, "fn();");
  let reparse = node_reparse(&reparse);

  assert_eq!(reparse.reparser, parser::Reparser::ModuleInner);
}

#[test]
fn eof_semicolon_repair_clears_stale_error() {
  let input = "root: mod\n";
  let old_parse = parse(input);
  assert!(!old_parse.errors().is_empty());

  let eof = input.len() as u32;
  let reparse = old_parse.reparse(
    input,
    &Indel {
      delete: TextRange::new(eof.into(), eof.into()),
      insert: ";".to_string(),
    },
  );
  let node = node_reparse(&reparse);
  let repaired = "root: mod\n;";

  assert_eq!(dump_parse(&node.parse), dump_parse(&parse(repaired)));
  assert!(node.parse.errors().is_empty());
  assert_eq!(node.reparser, parser::Reparser::ModuleInner);
}

#[test]
fn block_reparse_matches_full_parse() {
  let input = "x: = { a; b; };";
  let start = input.find("b").unwrap() as u32;

  reparse_eq_full(input, start..start + 1, "c");
}

#[test]
fn block_reparse_recovers_when_stmt_makes_no_progress() {
  let input = "x: = { a; };";
  let start = input.find("a").unwrap() as u32;

  reparse_eq_full(input, start..start + 1, ",");
}

#[test]
fn semicolon_repair_reparses_block_and_clears_stale_errors() {
  let input = "f: fn = { x: = 1; x };";
  let semi = input.find("; x").unwrap() as u32;
  let broken = reparse_eq_full(input, semi..semi + 1, "");
  let broken_node = node_reparse(&broken);

  assert_eq!(broken_node.reparser, parser::Reparser::BraceExpr);
  assert!(!broken_node.parse.errors().is_empty());

  let broken_text = "f: fn = { x: = 1 x };";
  let repair = broken_node.parse.reparse(
    broken_text,
    &Indel {
      delete: TextRange::new(semi.into(), semi.into()),
      insert: ";".to_string(),
    },
  );
  let repair_node = node_reparse(&repair);
  let repaired_text = "f: fn = { x: = 1; x };";

  assert_eq!(
    dump_parse(&repair_node.parse),
    dump_parse(&parse(repaired_text))
  );
  assert!(repair_node.parse.errors().is_empty());
  assert_eq!(repair_node.reparser, parser::Reparser::BraceExpr);
}

#[test]
fn thin_arrow_repair_uses_real_block_delimiters() {
  let input = "outer: fn = {\n  inner: fn -> () = {};\n};";
  let minus = input.find("->").unwrap() as u32;
  let broken = reparse_eq_full(input, minus..minus + 1, "");
  let broken_node = node_reparse(&broken);

  let broken_text = "outer: fn = {\n  inner: fn > () = {};\n};";
  let repair = broken_node.parse.reparse(
    broken_text,
    &Indel {
      delete: TextRange::new(minus.into(), minus.into()),
      insert: "-".to_string(),
    },
  );
  let repair_node = node_reparse(&repair);
  let repaired_text = "outer: fn = {\n  inner: fn -> () = {};\n};";

  assert_eq!(
    dump_parse(&repair_node.parse),
    dump_parse(&parse(repaired_text))
  );
  assert!(repair_node.parse.errors().is_empty());
  assert_eq!(repair_node.reparser, parser::Reparser::BraceExpr);
  assert_eq!(
    &broken_text
      [usize::from(repair_node.dirty.old.start())..usize::from(repair_node.dirty.old.end())],
    "{\n  inner: fn > () = {};\n}"
  );
}

#[test]
fn brace_reparse_allows_empty_struct_to_field_struct() {
  let input = "test: = {\n  \n};";
  let insert_at = input.find("  \n").unwrap() as u32 + 2;
  let reparse = reparse_eq_full(input, insert_at..insert_at, "a: 1");
  let reparse = node_reparse(&reparse);

  assert!(reparse.parse.errors().is_empty());
  assert_eq!(reparse.reparser, parser::Reparser::BraceExpr);
  assert_eq!(reparse.old_kind, SyntaxKind::StructExpr);
  assert_eq!(reparse.new_kind, SyntaxKind::StructExpr);
}

#[test]
fn brace_reparse_allows_empty_struct_to_shorthand_struct() {
  let input = "test: = {\n  \n};";
  let insert_at = input.find("  \n").unwrap() as u32 + 2;
  let reparse = reparse_eq_full(input, insert_at..insert_at, "a");
  let reparse = node_reparse(&reparse);

  assert!(reparse.parse.errors().is_empty());
  assert_eq!(reparse.reparser, parser::Reparser::BraceExpr);
  assert_eq!(reparse.old_kind, SyntaxKind::StructExpr);
  assert_eq!(reparse.new_kind, SyntaxKind::StructExpr);
}

#[test]
fn struct_field_value_token_join_repair_clears_stale_errors() {
  let input = "test: = {\n  key: val 111\n};";
  let space = input.find("val 111").unwrap() as u32 + 3;
  let reparse = reparse_eq_full(input, space..space + 1, "");
  let reparse = node_reparse(&reparse);

  assert!(reparse.parse.errors().is_empty());
  assert_eq!(reparse.reparser, parser::Reparser::BraceExpr);
  assert_eq!(reparse.old_kind, SyntaxKind::StructExpr);
  assert_eq!(reparse.new_kind, SyntaxKind::StructExpr);
}

#[test]
fn brace_reparse_allows_block_to_struct_mutation() {
  let input = "test: = {\n  a;\n};";
  let start = input.find("a;").unwrap() as u32;
  let reparse = reparse_eq_full(input, start..start + 2, "a: 1");
  let reparse = node_reparse(&reparse);

  assert!(reparse.parse.errors().is_empty());
  assert_eq!(reparse.reparser, parser::Reparser::BraceExpr);
  assert_eq!(reparse.old_kind, SyntaxKind::BlockExpr);
  assert_eq!(reparse.new_kind, SyntaxKind::StructExpr);
}

#[test]
fn balanced_delimiter_edit_reparses_index_arg() {
  let input = "x: = a[i];";
  let end = input.find("];").unwrap() as u32;
  let reparse = reparse_eq_full(input, end..end, " + f()");
  let reparse = node_reparse(&reparse);

  assert_eq!(reparse.reparser, parser::Reparser::IndexArg);
}

#[test]
fn unbalanced_delimiter_edit_rejects_incremental_reparse() {
  let input = "x: = { a; };";
  let parse = parse(input);
  let offset = input.find("a").unwrap() as u32;
  let reparse = parse.reparse(
    input,
    &Indel {
      delete: TextRange::new(offset.into(), offset.into()),
      insert: "{".to_string(),
    },
  );

  assert!(matches!(reparse, Reparse::Full(_)));
}

#[test]
fn module_inner_reparse_matches_full_parse() {
  let input = "foo: mod = { a: Int; };";
  let insert_at = input.find("};").unwrap() as u32;

  reparse_eq_full(input, insert_at..insert_at, " b: fn();");
}

#[test]
fn reparse_rejects_shebang_region() {
  let input = "#!/usr/bin/env autolang\nfoo: mod;";
  let parse = parse(input);
  let indel = Indel {
    delete: TextRange::new(0.into(), 2.into()),
    insert: "##".to_string(),
  };

  assert!(matches!(parse.reparse(input, &indel), Reparse::Full(_)));
}

#[test]
fn file_start_reparse_lexes_shebang() {
  let input = "foo: mod;";
  let shebang = "#!/usr/bin/env autolang\n";
  let parse = parse(input);

  let reparse = parse.reparse(
    input,
    &Indel {
      delete: TextRange::new(0.into(), 0.into()),
      insert: shebang.to_string(),
    },
  );

  assert!(matches!(reparse, Reparse::Full(_)));
}
