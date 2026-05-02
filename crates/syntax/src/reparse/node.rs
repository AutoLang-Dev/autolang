use super::{DirtyRange, Indel, NodeReparse, errors::merge_errors, range::map_containing_range};
use crate::{GreenNode, Lang, NodeOrToken, Parse, SyntaxKind, SyntaxNode, SyntaxToken, TextRange};
use rowan::Language;

use super::range::range_text;

pub fn reparse_node(
  parse: &Parse,
  old_text: &str,
  new_text: &str,
  indel: &Indel,
) -> Option<NodeReparse> {
  let root = parse.syntax_node();
  let element = if indel.delete.is_empty() {
    root
      .token_at_offset(indel.delete.start())
      .left_biased()
      .map(NodeOrToken::Token)?
  } else {
    root.covering_element(indel.delete)
  };

  let mut node = match &element {
    NodeOrToken::Node(node) => Some(node.clone()),
    NodeOrToken::Token(token) => token.parent(),
  };
  while let Some(candidate) = node {
    if let Some(reparser) = parser::reparser(candidate.kind())
      && !touches_own_delimiter(&candidate, indel)
      && let Some(reparse) =
        try_reparse_node(parse, old_text, new_text, indel, &candidate, reparser)
    {
      return Some(reparse);
    }
    node = candidate.parent();
  }

  None
}

fn touches_own_delimiter(node: &SyntaxNode, indel: &Indel) -> bool {
  let Some((open, close)) = own_delimiters(node) else {
    return false;
  };

  range_touches_token(indel.delete, &open) || range_touches_token(indel.delete, &close)
}

fn own_delimiters(node: &SyntaxNode) -> Option<(SyntaxToken, SyntaxToken)> {
  match node.kind() {
    SyntaxKind::ArgList
    | SyntaxKind::ArrayExpr
    | SyntaxKind::ArrayType
    | SyntaxKind::BlockExpr
    | SyntaxKind::DelimitedTokenTree
    | SyntaxKind::IndexArg
    | SyntaxKind::ParameterList
    | SyntaxKind::ParenExpr
    | SyntaxKind::ParenType
    | SyntaxKind::RepeatExpr
    | SyntaxKind::SliceType
    | SyntaxKind::StructExpr
    | SyntaxKind::StructType
    | SyntaxKind::TupleExpr
    | SyntaxKind::TupleType
    | SyntaxKind::UsingTreeList => (),
    _ => return None,
  }

  let open = node.first_token()?;
  let close = node.last_token()?;
  is_delimiter_pair(open.kind(), close.kind()).then_some((open, close))
}

fn is_delimiter_pair(open: SyntaxKind, close: SyntaxKind) -> bool {
  matches!(
    (open, close),
    (SyntaxKind::OpenParen, SyntaxKind::CloseParen)
      | (SyntaxKind::OpenBrack, SyntaxKind::CloseBrack)
      | (SyntaxKind::OpenBrace, SyntaxKind::CloseBrace)
  )
}

fn range_touches_token(range: TextRange, token: &SyntaxToken) -> bool {
  if range.is_empty() {
    return false;
  }

  range.intersect(token.text_range()).is_some()
}

fn try_reparse_node(
  parse: &Parse,
  old_text: &str,
  new_text: &str,
  indel: &Indel,
  old_node: &SyntaxNode,
  reparser: parser::Reparser,
) -> Option<NodeReparse> {
  let old_dirty_range = old_node.text_range();
  if !old_dirty_range.contains_range(indel.delete) {
    return None;
  }

  let new_dirty_range = map_containing_range(old_dirty_range, indel)?;
  let fragment = range_text(new_text, new_dirty_range);
  let fragment_parse = parse_reparse_fragment(reparser, fragment)?;
  let fragment_root = fragment_parse.syntax_node();

  if !reparse_kind_matches(reparser, old_node.kind(), fragment_root.kind()) {
    return None;
  }
  if own_delimiters(old_node).is_some() && own_delimiters(&fragment_root).is_none() {
    return None;
  }

  if range_text(old_text, old_dirty_range) == range_text(new_text, new_dirty_range) {
    return None;
  }

  let dirty = DirtyRange {
    old: old_dirty_range,
    new: new_dirty_range,
  };
  build_node_reparse(
    parse,
    old_node,
    fragment_parse,
    dirty,
    indel.delta(),
    reparser,
    fragment_root.kind(),
  )
}

fn parse_reparse_fragment(reparser: parser::Reparser, fragment: &str) -> Option<Parse> {
  let lexed = parser::LexedStr::relex(fragment);
  let output = reparser.parse(&lexed);
  crate::parse::parse_fragment(&lexed, &output)
}

fn build_node_reparse(
  parse: &Parse,
  old_node: &SyntaxNode,
  fragment_parse: Parse,
  dirty: DirtyRange,
  delta: i64,
  reparser: parser::Reparser,
  new_kind: SyntaxKind,
) -> Option<NodeReparse> {
  let old_kind = old_node.kind();
  let green = replace_node(old_node, fragment_parse.green.clone())?;
  let errors = merge_errors(
    parse.errors(),
    dirty.old,
    dirty.new,
    delta,
    fragment_parse.errors(),
    true,
  );

  Some(NodeReparse {
    parse: Parse { green, errors },
    dirty,
    reparser,
    old_kind,
    new_kind,
  })
}

fn replace_node(old_node: &SyntaxNode, new_green: GreenNode) -> Option<GreenNode> {
  if old_node.kind() == Lang::kind_from_raw(new_green.kind()) {
    return Some(old_node.replace_with(new_green));
  }

  let parent = old_node.parent()?;
  let new_parent = parent
    .green()
    .replace_child(old_node.index(), NodeOrToken::Node(new_green));
  Some(parent.replace_with(new_parent))
}

fn reparse_kind_matches(
  reparser: parser::Reparser,
  old_kind: SyntaxKind,
  new_kind: SyntaxKind,
) -> bool {
  match reparser {
    parser::Reparser::BraceExpr => matches!(
      (old_kind, new_kind),
      (
        SyntaxKind::BlockExpr | SyntaxKind::StructExpr,
        SyntaxKind::BlockExpr | SyntaxKind::StructExpr
      )
    ),
    parser::Reparser::ArrayExpr => matches!(
      (old_kind, new_kind),
      (
        SyntaxKind::ArrayExpr | SyntaxKind::RepeatExpr,
        SyntaxKind::ArrayExpr | SyntaxKind::RepeatExpr
      )
    ),
    parser::Reparser::ArrayOrSliceType => matches!(
      (old_kind, new_kind),
      (
        SyntaxKind::ArrayType | SyntaxKind::SliceType,
        SyntaxKind::ArrayType | SyntaxKind::SliceType
      )
    ),
    parser::Reparser::TupleOrParenExpr => matches!(
      (old_kind, new_kind),
      (
        SyntaxKind::ParenExpr | SyntaxKind::TupleExpr,
        SyntaxKind::ParenExpr | SyntaxKind::TupleExpr
      )
    ),
    parser::Reparser::TupleOrParenType => matches!(
      (old_kind, new_kind),
      (
        SyntaxKind::ParenType | SyntaxKind::TupleType,
        SyntaxKind::ParenType | SyntaxKind::TupleType
      )
    ),
    _ => old_kind == new_kind,
  }
}
