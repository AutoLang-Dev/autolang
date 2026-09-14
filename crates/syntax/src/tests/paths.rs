//! Path accessors, in particular the leading `::`.

use super::parse_red;
use crate::ast::{self, Node};

/// Every path of a file, in source order.
fn paths(text: &str) -> Vec<ast::Path> {
  let root = ast::Root::new(parse_red(text)).expect("a source file");

  root
    .red()
    .descendants()
    .filter_map(ast::Path::new)
    .collect()
}

/// A path as it was written, taken straight from the text its segments span.
fn render(path: &ast::Path, text: &str) -> String {
  let mut rendered = String::new();

  if path.global() {
    rendered.push_str("::");
  }
  for (index, segment) in path.segments().iter().enumerate() {
    if index > 0 {
      rendered.push_str("::");
    }
    rendered.push_str(&text[segment.red().range()]);
  }

  rendered
}

#[test]
fn a_leading_colon_colon_makes_a_path_global() {
  let text = "#[::a::b] foo: mod;\n";
  let paths = paths(text);

  assert_eq!(paths.len(), 1);
  assert!(paths[0].global());
  assert_eq!(render(&paths[0], text), "::a::b");
}

/// `a::b` also contains a `::`, but it does not start with one.
#[test]
fn a_path_without_a_leading_colon_colon_is_relative() {
  let text = "foo: type = a::b;\n";
  let paths = paths(text);

  assert_eq!(paths.len(), 1);
  assert!(!paths[0].global());
  assert_eq!(render(&paths[0], text), "a::b");
}

#[test]
fn a_using_list_prefix_is_global_while_its_leaves_are_relative() {
  let text = "#[::a] foo: mod = { b: mod; };\n";
  let paths = paths(text);

  // The `_` leaf is not a path, so only four are collected.
  assert_eq!(
    paths
      .iter()
      .map(|path| render(path, text))
      .collect::<Vec<_>>(),
    ["::a"]
  );
  assert!(paths[0].global(), "the prefix of the list is global");
  assert!(
    paths[1..].iter().all(|path| !path.global()),
    "the leaves are written relative to the prefix"
  );
}

#[test]
fn keywords_are_segments_like_any_other() {
  let text = "#[unit] foo: mod;\n";
  let paths = paths(text);

  assert_eq!(
    paths
      .iter()
      .map(|path| render(path, text))
      .collect::<Vec<_>>(),
    ["unit"]
  );
}
