use crate::parse;

macro_rules! parse_snap {
  ($input:literal) => {{
    let input = $input;
    snap!(input, parse(input));
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
fn underscore_mod_decl() {
  parse_snap!(r#"_: mod;"#);
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
  parse_snap!(r#"x: Int = 42;"#);
}

#[test]
fn binding_without_type() {
  parse_snap!(r#"x: = 42;"#);
}

#[test]
fn binding_patterns() {
  parse_snap!(r#"_: Int = 0; mut x: Int = 1;"#);
}

#[test]
fn function_item() {
  parse_snap!(r#"add: fn(a: Int, b: Int) -> Int = a + b;"#);
}

#[test]
fn nested_expr_delimiters() {
  parse_snap!(r#"x: = ({ a; }, [b, (c)]);"#);
}

#[test]
fn expr_operator_precedence() {
  parse_snap!(r#"x: = a + b * c == d && e;"#);
}

#[test]
fn logical_chain_expr() {
  parse_snap!(r#"test: = 1 && 2 && 3;"#);
}

#[test]
fn comparison_chain_expr() {
  parse_snap!(r#"x: = a == b < c != d;"#);
}

#[test]
fn normal_binary_expr_stays_nested() {
  parse_snap!(r#"x: = a + b + c;"#);
}

#[test]
fn expr_prefix_postfix_call_index_cast_field() {
  parse_snap!(r#"x: = -foo.bar(1)[i]++ as Int;"#);
}

#[test]
fn path_call_with_colon_colon_in_block() {
  parse_snap!(r#"x: = { Response::json(saved); { Response::ok } };"#);
}

#[test]
fn expr_control_flow_atoms() {
  parse_snap!(r#"x: = { return a; break 'done b; cont c; };"#);
}

#[test]
fn block_statements_and_tail_expr() {
  parse_snap!(r#"x: = { y: Int = 1; y += 2; y; y } ;"#);
}

#[test]
fn control_flow_exprs() {
  parse_snap!(
    r#"x: = { if ready { run } else { stop }; while cond { step } else { done }; for mut x in xs { x }; iterate acc = init { cont acc }; case { a = b, c = d } };"#
  );
}

#[test]
fn remaining_expr_forms() {
  parse_snap!(
    r#"x: = { { name, other }; [value; count]; fn(a: Int) -> Int = a; recv.method(arg); 'loop: while cond { break 'loop done } };"#
  );
}

#[test]
fn struct_expr_disambiguation() {
  parse_snap!(
    r#"x: = { { name: value }; { nested: { other: value } }; { name: (a + b), other: [x; y] } };"#
  );
}

#[test]
fn struct_expr_field_recovery_keeps_close_brace() {
  parse_snap!(r#"x: = { key: val 111 };"#);
}

#[test]
fn block_expr_disambiguation() {
  parse_snap!(r#"x: = { { y: Int; }; { y: Int = 1; }; { y: = 1; }; { y += 1; } };"#);
}

#[test]
fn block_stmt_recovery_makes_progress() {
  parse_snap!(r#"x: = { , };"#);
}

#[test]
fn missing_binding_expr() {
  parse_snap!(r#"x: = ;"#);
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
  parse_snap!(r#"F: type = (Int, &mut User) mut -> [*Int; 4]; xs: [Int] = data;"#);
}

#[test]
fn attrs_and_visibility() {
  parse_snap!(
    r#"#[entry] pub app: fn(input: Int) -> Int = { #[cold] run; }; Point: type = { #[x] pub x: Int, pri y: Int };"#
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
  parse_snap!(
    r#"foo: mod = { x: Int = 42; add: fn(a: Int) -> Int = a; Point: type = { x: Int }; bar: mod; };"#
  );
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
