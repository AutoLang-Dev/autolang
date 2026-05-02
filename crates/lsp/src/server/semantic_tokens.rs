use crate::server::{Document, Server};
use line_index::WideEncoding;
use lsp_types::{
  SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensDelta, SemanticTokensEdit,
  SemanticTokensFullDeltaResult, SemanticTokensLegend, Uri,
};
use std::sync::atomic::{AtomicU64, Ordering};
use syntax::SyntaxKind;

static NEXT_RESULT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Default)]
pub struct SemanticCache {
  result_id: Option<String>,
  spans: Vec<SemanticSpan>,
  data: Vec<SemanticToken>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SemanticDirty {
  #[default]
  Clean,
  Precise {
    old_range: syntax::TextRange,
    new_range: syntax::TextRange,
  },
  Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticSpan {
  range: syntax::TextRange,
  len: u32,
  token_type: u32,
  token_modifiers_bitset: u32,
}

macro_rules! define_tokens {
  ( $($name:ident => $token:ident),* $(,)? ) => {
    #[repr(u32)]
    pub enum TokenType {
      $(
        $name,
      )*
    }

    pub fn tokens_legend() -> SemanticTokensLegend {
      SemanticTokensLegend {
        token_types: vec![
          $(SemanticTokenType::$token, )*
        ],
        token_modifiers: vec![],
      }
    }
  };
}

define_tokens! {
  Comment => COMMENT,
  Keyword => KEYWORD,
  Module => NAMESPACE,
  Function => FUNCTION,
  Type => TYPE,
  Parameter => PARAMETER,
  Property => PROPERTY,
  Decorator => DECORATOR,
  Modifier => MODIFIER,
  Ident => VARIABLE,
  String => STRING,
  Number => NUMBER,
  Operator => OPERATOR,
  Label => MACRO,
}

fn map_token(token: &syntax::SyntaxToken) -> Option<u32> {
  if let Some(token_type) = map_context_token(token) {
    return Some(token_type as u32);
  }

  if let Some(token_type) = map_name_token(token) {
    return Some(token_type as u32);
  }

  map_token_kind(token.kind())
}

fn map_token_kind(kind: SyntaxKind) -> Option<u32> {
  let tt = match kind {
    SyntaxKind::Whitespace => return None,
    SyntaxKind::Comment | SyntaxKind::Shebang => TokenType::Comment,
    SyntaxKind::Ident | SyntaxKind::Underscore => TokenType::Ident,
    SyntaxKind::Label => TokenType::Label,
    SyntaxKind::Int => TokenType::Number,
    SyntaxKind::Char | SyntaxKind::Byte | SyntaxKind::String | SyntaxKind::RawString => {
      TokenType::String
    }
    kind if kind.is_keyword() => TokenType::Keyword,
    kind if kind.is_punct() => TokenType::Operator,
    _ => return None,
  };

  Some(tt as u32)
}

fn map_context_token(token: &syntax::SyntaxToken) -> Option<TokenType> {
  let parent = token.parent()?;

  match parent.kind() {
    SyntaxKind::Visibility
      if matches!(
        token.kind(),
        SyntaxKind::KwPub | SyntaxKind::KwPro | SyntaxKind::KwPri
      ) =>
    {
      Some(TokenType::Modifier)
    }
    _ => None,
  }
}

fn map_name_token(token: &syntax::SyntaxToken) -> Option<TokenType> {
  if !matches!(token.kind(), SyntaxKind::Ident | SyntaxKind::Underscore) {
    return None;
  }

  let parent = token.parent()?;

  match parent.kind() {
    SyntaxKind::Rename => Some(TokenType::Module),
    SyntaxKind::ModuleItem if is_name_before_colon(token, &parent) => Some(TokenType::Module),
    SyntaxKind::FunctionItem if is_name_before_colon(token, &parent) => Some(TokenType::Function),
    SyntaxKind::TypeItem if is_name_before_colon(token, &parent) => Some(TokenType::Type),
    SyntaxKind::BindingItem if is_name_before_colon(token, &parent) => Some(TokenType::Ident),
    _ if is_parameter_name(token, &parent) => Some(TokenType::Parameter),
    SyntaxKind::FieldName => Some(TokenType::Property),
    _ if is_attr_path_token(&parent) => Some(TokenType::Decorator),
    _ if is_using_tree_token(&parent) => Some(TokenType::Module),
    _ if is_type_path_token(&parent) => Some(TokenType::Type),
    _ => None,
  }
}

fn is_name_before_colon(token: &syntax::SyntaxToken, parent: &syntax::SyntaxNode) -> bool {
  parent
    .children_with_tokens()
    .filter_map(|element| element.into_token())
    .take_while(|sibling| sibling.kind() != SyntaxKind::Colon)
    .find(|sibling| matches!(sibling.kind(), SyntaxKind::Ident | SyntaxKind::Underscore))
    .is_some_and(|name| name == *token)
}

fn is_parameter_name(token: &syntax::SyntaxToken, parent: &syntax::SyntaxNode) -> bool {
  let Some(parameter) = parent
    .ancestors()
    .find(|node| node.kind() == SyntaxKind::Parameter)
  else {
    return false;
  };

  parameter
    .descendants_with_tokens()
    .filter_map(|element| element.into_token())
    .take_while(|sibling| sibling.kind() != SyntaxKind::Colon)
    .find(|sibling| matches!(sibling.kind(), SyntaxKind::Ident | SyntaxKind::Underscore))
    .is_some_and(|name| name == *token)
}

fn is_type_path_token(parent: &syntax::SyntaxNode) -> bool {
  parent
    .ancestors()
    .any(|node| node.kind() == SyntaxKind::PathType)
}

fn is_attr_path_token(parent: &syntax::SyntaxNode) -> bool {
  parent
    .ancestors()
    .any(|node| node.kind() == SyntaxKind::Attr)
    && !parent
      .ancestors()
      .any(|node| node.kind() == SyntaxKind::AttrArg)
}

fn is_using_tree_token(parent: &syntax::SyntaxNode) -> bool {
  parent
    .ancestors()
    .any(|node| node.kind() == SyntaxKind::UsingTree)
}

impl Server {
  pub fn semantic_tokens(&mut self, uri: &Uri) -> SemanticTokens {
    let Some(document) = self.documents.get_mut(uri) else {
      return SemanticTokens {
        result_id: None,
        data: vec![],
      };
    };

    document.semantic_tokens_full()
  }

  pub fn semantic_tokens_delta(
    &mut self,
    uri: &Uri,
    previous_result_id: &str,
  ) -> SemanticTokensFullDeltaResult {
    let Some(document) = self.documents.get_mut(uri) else {
      return SemanticTokensFullDeltaResult::Tokens(SemanticTokens {
        result_id: None,
        data: vec![],
      });
    };

    document.semantic_tokens_delta(previous_result_id)
  }
}

impl Document {
  fn semantic_tokens_full(&mut self) -> SemanticTokens {
    let spans = self.semantic_spans();
    let data = self.encode_spans(&spans, None);
    let result_id = next_result_id();

    self.semantic_cache = SemanticCache {
      result_id: Some(result_id.clone()),
      spans,
      data: data.clone(),
    };
    self.semantic_dirty = SemanticDirty::Clean;

    SemanticTokens {
      result_id: Some(result_id),
      data,
    }
  }

  fn semantic_tokens_delta(&mut self, previous_result_id: &str) -> SemanticTokensFullDeltaResult {
    if self.semantic_cache.result_id.as_deref() != Some(previous_result_id)
      || matches!(self.semantic_dirty, SemanticDirty::Full)
    {
      return SemanticTokensFullDeltaResult::Tokens(self.semantic_tokens_full());
    }

    let SemanticDirty::Precise {
      old_range,
      new_range,
    } = self.semantic_dirty.clone()
    else {
      let result_id = next_result_id();
      self.semantic_cache.result_id = Some(result_id.clone());
      return SemanticTokensFullDeltaResult::TokensDelta(SemanticTokensDelta {
        result_id: Some(result_id),
        edits: vec![],
      });
    };

    let old_spans = self.semantic_cache.spans.clone();
    debug_assert_eq!(self.semantic_cache.data.len(), old_spans.len());
    let new_spans = self.semantic_spans();
    let new_data = self.encode_spans(&new_spans, None);
    let old_range_tokens = token_range_for_dirty(&old_spans, old_range);
    let new_range_tokens = token_range_for_dirty(&new_spans, new_range);
    let old_range_tokens = extend_with_next_token(old_range_tokens, old_spans.len());
    let new_range_tokens = extend_with_next_token(new_range_tokens, new_spans.len());

    let prev_span = new_range_tokens
      .start
      .checked_sub(1)
      .and_then(|idx| new_spans.get(idx));
    let data = self.encode_spans(&new_spans[new_range_tokens.clone()], prev_span);
    let result_id = next_result_id();
    let edit = SemanticTokensEdit {
      start: (old_range_tokens.start * 5) as u32,
      delete_count: ((old_range_tokens.end - old_range_tokens.start) * 5) as u32,
      data: Some(data),
    };

    self.semantic_cache = SemanticCache {
      result_id: Some(result_id.clone()),
      spans: new_spans,
      data: new_data,
    };
    self.semantic_dirty = SemanticDirty::Clean;

    SemanticTokensFullDeltaResult::TokensDelta(SemanticTokensDelta {
      result_id: Some(result_id),
      edits: vec![edit],
    })
  }

  fn semantic_spans(&self) -> Vec<SemanticSpan> {
    let root = self.parse.syntax_node();
    let mut spans = Vec::new();

    for token in root
      .descendants_with_tokens()
      .filter_map(|element| element.into_token())
    {
      let Some(token_type) = map_token(&token) else {
        continue;
      };

      let range = token.text_range();
      let text = token.text();

      spans.push(SemanticSpan {
        range,
        len: WideEncoding::Utf16.measure(text) as u32,
        token_type,
        token_modifiers_bitset: 0,
      });
    }

    spans
  }

  fn encode_spans(
    &self,
    spans: &[SemanticSpan],
    prev: Option<&SemanticSpan>,
  ) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let (mut prev_line, mut prev_col) = prev
      .map(|span| {
        let pos = self.position(span.range.start());
        (pos.line, pos.character)
      })
      .unwrap_or((0, 0));

    for span in spans {
      let line_col = self.position(span.range.start());
      let start_col = if line_col.line == prev_line {
        prev_col
      } else {
        0
      };

      tokens.push(SemanticToken {
        delta_line: line_col.line - prev_line,
        delta_start: line_col.character - start_col,
        length: span.len,
        token_type: span.token_type,
        token_modifiers_bitset: span.token_modifiers_bitset,
      });

      prev_line = line_col.line;
      prev_col = line_col.character;
    }

    tokens
  }
}

fn token_range_for_dirty(
  spans: &[SemanticSpan],
  dirty: syntax::TextRange,
) -> std::ops::Range<usize> {
  let start = spans
    .iter()
    .position(|span| span.range.end() > dirty.start())
    .unwrap_or(spans.len());
  let end = spans
    .iter()
    .position(|span| span.range.start() >= dirty.end())
    .unwrap_or(spans.len());
  start..end.max(start)
}

fn extend_with_next_token(range: std::ops::Range<usize>, len: usize) -> std::ops::Range<usize> {
  range.start..(range.end + usize::from(range.end < len))
}

fn next_result_id() -> String {
  NEXT_RESULT_ID.fetch_add(1, Ordering::Relaxed).to_string()
}
