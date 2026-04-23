use crate::lex;
use insta::{assert_debug_snapshot, with_settings};

macro_rules! debug {
  ($input:literal) => {
    let input = $input;
    with_settings!({
      description => input,
    }, {
      assert_debug_snapshot!(lex(input).collect::<Vec<_>>());
    });
  };
}

#[test]
fn simple_binding() {
  debug!(r#"x: Int = 42;"#);
}

#[test]
fn function_definition() {
  debug!(r#"add: fn(a: Int, b: Int) -> Int = a + b;"#);
}

#[test]
fn comment_and_whitespace() {
  debug!(
    r#"// this is a comment
    x: Int = 1;"#
  );
}

#[test]
fn string_literals() {
  debug!(r#""hello" "world""#);
}

#[test]
fn string_with_escape() {
  debug!(r#""hello\nworld""#);
}

#[test]
fn string_with_comment_inside() {
  debug!(r#""hello // world""#);
}

#[test]
fn multiline_string() {
  debug!(
    r#""line1
"line2"#
  );
}

#[test]
fn char_and_byte_literals() {
  debug!(r#"'a' b'x' b"hi""#);
}

#[test]
fn raw_string() {
  debug!(r#"''raw string here"#);
}

#[test]
fn integer_literals() {
  debug!(r#"42 0b1010 0o77 0xFF 0rz233 114'514"#);
}

#[test]
fn any_radix_integer() {
  debug!(r#"0r2'01 0r8'765 0ra123 0rb456"#);
}

#[test]
fn labels() {
  debug!(r#"'loop 'end"#);
}

#[test]
fn path_and_attributes() {
  debug!(r#"::std::io #[attr] pub"#);
}

#[test]
fn operators_in_context() {
  debug!(r#"x += 1; y >>= 2;"#);
}

#[test]
fn comparison_and_logic() {
  debug!(r#"a == b && c != d || e"#);
}

#[test]
fn if_block() {
  debug!(r#"if x > 0 { return x; }"#);
}

#[test]
fn for_loop() {
  debug!(r#"for x in list { print(x); }"#);
}

#[test]
fn type_definition() {
  debug!(r#"Point: type = { x: Int, y: Int };"#);
}

#[test]
fn unicode_identifiers() {
  debug!(r#"café 变量"#);
}

#[test]
fn unknown_character() {
  debug!(r#"x ` y"#);
}

#[test]
fn empty_input() {
  debug!(r#""#);
}

#[test]
fn complex_program() {
  debug!(
    r#"main: fn() = {
  x: Int = 42;
  y: String = "hello";
  // compute
  result: Int = x + 1;
  if result > 40 {
    return result;
  }
};"#
  );
}
