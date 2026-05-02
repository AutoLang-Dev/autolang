use crate::{NodeOrToken, Parse};
use rowan::WalkEvent;

pub fn syntax_tree(parse: &Parse) -> String {
  let mut lines = syntax_tree_lines(parse);

  if !parse.errors().is_empty() {
    lines.push(String::new());
    lines.push("Errors:".to_string());
    for error in parse.errors() {
      lines.push(format!("  {error:?}"));
    }
  }

  lines.join("\n")
}

pub(crate) fn syntax_tree_lines(parse: &Parse) -> Vec<String> {
  let root = parse.syntax_node();
  let mut lines = Vec::new();
  let mut level = 0;

  for event in root.preorder_with_tokens() {
    match event {
      WalkEvent::Enter(NodeOrToken::Node(node)) => {
        lines.push(format!("{}{:?}", "  ".repeat(level), node.kind()));
        level += 1;
      }
      WalkEvent::Leave(NodeOrToken::Node(_)) => level -= 1,
      WalkEvent::Enter(NodeOrToken::Token(token)) => lines.push(format!(
        "{}{:?} {:?}",
        "  ".repeat(level),
        token.kind(),
        token.text()
      )),
      WalkEvent::Leave(NodeOrToken::Token(_)) => (),
    }
  }

  lines
}
