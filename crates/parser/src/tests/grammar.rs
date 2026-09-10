use crate::{LexedStr, Output};

pub fn parse(text: &str) -> Output {
  let lexed = LexedStr::new(text);
  crate::parse(&lexed)
}

pub fn parse_expr(text: &str) -> Output {
  let lexed = LexedStr::new(text);
  crate::parse_expr(&lexed)
}

macro_rules! parse_snap {
  ($input:literal) => {{
    let input = $input;
    snap!(input, parse(input));
  }};
}

macro_rules! parse_expr_snap {
  ($input:literal) => {{
    let input = $input;
    snap!(input, parse_expr(input));
  }};
}

#[test]
fn empty_input() {
  parse_snap!(r#""#);
}

#[test]
fn mod_decl() {
  parse_snap!(r#"foo: mod;"#);
}

#[test]
fn underscore_mod_decl_is_rejected() {
  parse_snap!(r#"_: mod;"#);
}

#[test]
fn names_are_wrapped_in_name_nodes() {
  // Item names, binding names, field names, path segments and rename targets
  // are all `Name(Ident)`; `as _` and `foo::_` keep a bare `Underscore`.
  parse_snap!(r#"using foo as _; using bar::{baz as _, qux}; a: type = self::unit;"#);
}

#[test]
fn raw_token_tree_idents_are_not_names() {
  parse_snap!(r#"#[inner(flag, nested({ raw [tree] }))] foo: mod;"#);
}

#[test]
fn tuple_field_access_by_name() {
  parse_expr_snap!(r#"tup._0"#);
}

#[test]
fn numeric_field_access_is_rejected() {
  parse_expr_snap!(r#"tup.0"#);
}

#[test]
fn numeric_field_name_is_rejected() {
  parse_snap!(r#"T: type = { 0: Int };"#);
}

#[test]
fn inline_mod() {
  parse_snap!(r#"foo: mod = { bar: mod; };"#);
}

#[test]
fn nested_inline_mod() {
  parse_snap!(r#"foo: mod = { bar: mod = { baz: mod; }; };"#);
}

#[test]
fn binding_item() {
  parse_expr_snap!(r#"{ let x: Int = 42; }"#);
}

#[test]
fn binding_without_type() {
  parse_expr_snap!(r#"{ x := 42; }"#);
}

#[test]
fn binding_patterns() {
  parse_expr_snap!(r#"{ let _: Int = 0; let mut x: Int = 1; }"#);
}

#[test]
fn binding_pattern_tuple() {
  parse_expr_snap!(r#"{ let (x, mut y) = (1, 2); }"#);
}

#[test]
fn binding_pattern_struct() {
  parse_expr_snap!(r#"{ let { x: mut a, y } = { x: 1, y: 2 }; }"#);
}

#[test]
fn function_item() {
  parse_snap!(r#"add: (Int, Int) -> Int = \(a, b). a + b;"#);
}

#[test]
fn nested_expr_delimiters() {
  parse_expr_snap!(r#"({ a; }, [b, (c)])"#);
}

#[test]
fn expr_operator_precedence() {
  parse_expr_snap!(r#"a + b * c == d /\ e"#);
}

#[test]
fn logical_chain_expr() {
  parse_expr_snap!(r#"1 /\ 2 /\ 3"#);
}

#[test]
fn comparison_chain_expr() {
  parse_expr_snap!(r#"a == b < c != d"#);
}

#[test]
fn normal_binary_expr_stays_nested() {
  parse_expr_snap!(r#"a + b + c"#);
}

#[test]
fn expr_prefix_postfix_call_index_cast_field() {
  parse_expr_snap!(r#"-foo.bar(1)[i]++ as Int"#);
}

#[test]
fn path_call_with_colon_colon_in_block() {
  parse_expr_snap!(r#"{ Response::json(saved); { Response::ok } }"#);
}

#[test]
fn expr_control_flow_atoms() {
  parse_expr_snap!(r#"{ return a; break 'done b; cont c; }"#);
}

#[test]
fn block_statements_and_tail_expr() {
  parse_expr_snap!(r#"{ y := 1; y }"#);
}

#[test]
fn control_flow_exprs() {
  parse_expr_snap!(
    r#"{ if ready { run } else { stop }; while cond { step } else { done }; for mut x in xs { x }; iterate acc = init { cont acc } }"#
  );
}

#[test]
fn case_is_a_regular_name() {
  // `case` is no longer a keyword, so it must lex as a plain identifier.
  parse_snap!(r#"case: mod = { case: mod; };"#);
}

#[test]
fn remaining_expr_forms() {
  parse_expr_snap!(
    r#"{ { name, other }; [value; count]; \a. a; recv.method(arg); 'loop: while cond { break 'loop done } }"#
  );
}

#[test]
fn record_expr_disambiguation() {
  parse_expr_snap!(
    r#"{ { name: value }; { nested: { other: value } }; { name: (a + b), other: [x; y] } }"#
  );
}

#[test]
fn record_expr_field_recovery_keeps_close_brace() {
  parse_expr_snap!(r#"{ key: val 111 }"#);
}

#[test]
fn tuple_expr() {
  parse_expr_snap!(r#"((), 114514, next: 1919810)"#);
}

#[test]
fn tuple_type() {
  parse_snap!(r#"T: type = (Int, next: Int);"#);
}

#[test]
fn missing_binding_expr() {
  parse_expr_snap!(r#"{ x := ; }"#);
}

#[test]
fn missing_type_item_rhs() {
  parse_snap!(r#"Point: type = ;"#);
}

#[test]
fn type_item() {
  parse_snap!(r#"Point: type = { x: Int, y: Int };"#);
}

#[test]
fn nominal_type_item() {
  parse_snap!(r#"UserId: nominal = Int;"#);
}

#[test]
fn complex_types() {
  parse_snap!(r#"F: type = &(Int, &mut User) mut -> [*Int; 4];"#);
}

#[test]
fn attrs_and_visibility() {
  parse_snap!(
    r#"#[entry] pub app: (Int) -> Int = \input. { #[cold] run; }; Point: type = { #[x] pub x: Int, pri y: Int };"#
  );
}

#[test]
fn inner_attrs_and_attr_args() {
  parse_snap!(
    r#"foo: mod = { #[inner(flag = true, nested({ a [b] }))]; #[outer = value] bar: mod; };"#
  );
}

#[test]
fn using_items() {
  parse_snap!(
    r#"using foo; using foo::bar as baz; using foo::_; using foo::{bar, baz as qux, nested::{one, two}}; #[prelude] pub using ::root::prelude;"#
  );
}

#[test]
fn mixed_module_inner() {
  parse_snap!(r#"foo: mod = { add: (Int) -> Int = \a. a; Point: type = { x: Int }; bar: mod; };"#);
}

#[test]
fn unexpected_item_recovery() {
  parse_snap!(r#"foo fn;"#);
}

#[test]
fn invalid_inner_attr_recovery() {
  parse_snap!(r#"foo: mod = { #![inner]; bar: mod; };"#);
}

#[test]
fn invalid_using_glob_recovery() {
  parse_snap!(r#"using foo::*; using bar;"#);
}

#[test]
fn condition_keeps_its_block() {
  // A brace call must never swallow the block that follows a head: neither
  // after a binary operator (`if a == b { c }`) nor after a prefix operator
  // (`if !a { c }`), nor anywhere inside a chain.
  parse_expr_snap!(
    r#"{ if a == b { c } else { d }; while a < b { c } else { d }; if !a { c }; if -a { c }; if &a { c }; if a /\ b { c } }"#
  );
}
