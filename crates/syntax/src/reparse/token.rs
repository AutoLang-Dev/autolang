use super::{DirtyRange, Indel, TokenReparse, errors::merge_errors, range::range_text};
use crate::{GreenToken, Lang, Parse, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, TextSize};
use rowan::Language;

pub fn reparse_token(parse: &Parse, old_text: &str, indel: &Indel) -> Option<TokenReparse> {
  if has_token_boundary_quote(range_text(old_text, indel.delete))
    || has_token_boundary_quote(&indel.insert)
  {
    return None;
  }

  let root = parse.syntax_node();
  for token in token_reparse_candidates(&root, indel) {
    if let Some(reparse) = try_reparse_token(parse, indel, token) {
      return Some(reparse);
    }
  }

  None
}

fn try_reparse_token(parse: &Parse, indel: &Indel, token: SyntaxToken) -> Option<TokenReparse> {
  if !token_allows_single_token_reparse(&token, indel) {
    return None;
  }

  let old_dirty_range = token.text_range();
  if !is_shape_safe_token(token.kind()) {
    return None;
  }

  let mut token_text = token.text().to_string();
  let rel_delete = TextRange::new(
    indel.delete.start() - old_dirty_range.start(),
    indel.delete.end() - old_dirty_range.start(),
  );
  Indel {
    delete: rel_delete,
    insert: indel.insert.clone(),
  }
  .apply(&mut token_text);

  let lexed = parser::LexedStr::relex(&token_text);
  let kind = single_lexed_token_kind(&lexed)?;
  if kind != token.kind() {
    return None;
  }

  let green = token.replace_with(GreenToken::new(Lang::kind_to_raw(kind), &token_text));
  let new_dirty_range = TextRange::at(old_dirty_range.start(), TextSize::of(&token_text));
  let errors = merge_errors(
    parse.errors(),
    old_dirty_range,
    new_dirty_range,
    indel.delta(),
    &[],
    false,
  );

  Some(TokenReparse {
    parse: Parse { green, errors },
    dirty: DirtyRange {
      old: old_dirty_range,
      new: new_dirty_range,
    },
  })
}

fn has_token_boundary_quote(text: &str) -> bool {
  text.contains(['"', '\''])
}

fn token_reparse_candidates(root: &SyntaxNode, indel: &Indel) -> Vec<SyntaxToken> {
  if !indel.delete.is_empty() {
    return root
      .covering_element(indel.delete)
      .into_token()
      .into_iter()
      .collect();
  }

  insertion_tokens(root, indel.delete.start())
}

fn token_allows_single_token_reparse(token: &SyntaxToken, indel: &Indel) -> bool {
  let range = token.text_range();
  if indel.delete.start() < range.start() || indel.delete.end() > range.end() {
    return false;
  }

  let touches_start = indel.delete.start() == range.start();
  let touches_end = indel.delete.end() == range.end();
  if !indel.delete.is_empty() && touches_start && touches_end {
    return false;
  }

  if !touches_start && !touches_end {
    return true;
  }

  if touches_start == touches_end {
    return false;
  }

  let adjacent = if touches_start {
    token.prev_token()
  } else {
    token.next_token()
  };
  let adjacent_is_trivia = adjacent
    .as_ref()
    .is_some_and(|token| token.kind().is_trivia());

  token.kind().is_trivia() || adjacent.is_none() || adjacent_is_trivia
}

fn insertion_tokens(root: &SyntaxNode, offset: TextSize) -> Vec<SyntaxToken> {
  let left = root.token_at_offset(offset).left_biased();
  let right = root.token_at_offset(offset).right_biased();
  let mut tokens = Vec::new();

  if let Some(token) = left {
    tokens.push(token);
  }
  if let Some(token) = right
    && !tokens
      .iter()
      .any(|prev: &SyntaxToken| prev.text_range() == token.text_range())
  {
    tokens.push(token);
  }

  tokens
}

fn single_lexed_token_kind(lexed: &parser::LexedStr<'_>) -> Option<SyntaxKind> {
  let mut kinds = (0..lexed.len()).map(|raw| lexed.kind(raw));
  let kind = kinds.next()?;
  kinds.next().is_none().then_some(kind)
}

fn is_shape_safe_token(kind: SyntaxKind) -> bool {
  kind.is_trivia()
    || matches!(
      kind,
      SyntaxKind::Ident
        | SyntaxKind::Underscore
        | SyntaxKind::Label
        | SyntaxKind::Int
        | SyntaxKind::Char
        | SyntaxKind::Byte
        | SyntaxKind::String
        | SyntaxKind::RawString
        | SyntaxKind::UnknownPrefix
        | SyntaxKind::Unknown
    )
}
