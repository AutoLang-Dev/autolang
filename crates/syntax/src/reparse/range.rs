use super::Indel;
use crate::{SyntaxKind, TextRange, TextSize};

pub fn delimiters_balanced(text: &str) -> bool {
  let mut stack = Vec::new();
  let lexed = parser::LexedStr::relex(text);
  for raw in 0..lexed.len() {
    match lexed.kind(raw) {
      SyntaxKind::OpenParen => stack.push(SyntaxKind::CloseParen),
      SyntaxKind::OpenBrack => stack.push(SyntaxKind::CloseBrack),
      SyntaxKind::OpenBrace => stack.push(SyntaxKind::CloseBrace),
      kind @ (SyntaxKind::CloseParen | SyntaxKind::CloseBrack | SyntaxKind::CloseBrace)
        if stack.pop() != Some(kind) =>
      {
        return false;
      }
      SyntaxKind::CloseParen | SyntaxKind::CloseBrack | SyntaxKind::CloseBrace => (),
      _ => (),
    }
  }
  stack.is_empty()
}

pub fn map_containing_range(range: TextRange, indel: &Indel) -> Option<TextRange> {
  if !range.contains_range(indel.delete) {
    return None;
  }

  let start = range.start();
  let end = add_delta(range.end(), indel.delta())?;
  Some(TextRange::new(start, end))
}

pub fn range_text(text: &str, range: TextRange) -> &str {
  &text[usize::from(range.start())..usize::from(range.end())]
}

pub fn add_delta(offset: TextSize, delta: i64) -> Option<TextSize> {
  let raw = i64::from(u32::from(offset)) + delta;
  (0..=i64::from(u32::MAX))
    .contains(&raw)
    .then(|| TextSize::from(raw as u32))
}
