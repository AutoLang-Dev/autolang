use crate::{Green, Indel, Payload, Red};
use parser::{LexedStr, SyntaxKind::*};
use text_size::TextSize;

pub fn reparse(token: &Red, source: &str, indel: &Indel) -> Option<Green> {
  if !safe_to_reparse(token, indel) {
    return None;
  }

  let rel_del = indel.delete - token.range().start();
  let rel_indel = Indel {
    delete: rel_del,
    insert: indel.insert.clone(),
  };

  let text = rel_indel.apply_to(&source[token.range()]);

  let lexed = LexedStr::new(&text);
  if lexed.len() != 1 {
    return None;
  }
  let kind = lexed.kind(0);
  if kind != token.kind() {
    return None;
  }

  Some(Green::token(
    kind,
    TextSize::of(&text),
    Payload { diag: None },
  ))
}

fn safe_to_reparse(token: &Red, indel: &Indel) -> bool {
  if !token.is_token() {
    return false;
  }

  let kind = token.kind();
  if !matches!(kind, Ident | Label | Int | Char | Byte | String | RawString) {
    return false;
  }

  let range = token.range();

  assert!(range.contains_range(indel.delete));

  let touch_start = indel.delete.start() == range.start();
  let touch_end = indel.delete.end() == range.end();

  let Some(adj) = (match (touch_start, touch_end) {
    (false, false) => return true,
    (true, true) => return false,
    (true, false) => token.prev_token(),
    (false, true) => token.next_token(),
  }) else {
    return true;
  };

  adj.kind().is_trivia() || kind.is_trivia()
}
