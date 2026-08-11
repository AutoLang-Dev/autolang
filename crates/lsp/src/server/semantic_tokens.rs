use crate::server::Server;
use ide::collect_semantic_tokens;
use line_index::WideEncoding;
use lsp_types::{
  Position, SemanticToken as LspSemanticToken, SemanticTokenType, SemanticTokens,
  SemanticTokensLegend, Uri,
};

macro_rules! define_tokens {
  ( $($name:ident => $token:ident),* $(,)? ) => {
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
  Field => PROPERTY,
  Method => METHOD,
  Decorator => DECORATOR,
  Modifier => MODIFIER,
  Ident => VARIABLE,
  String => STRING,
  Number => NUMBER,
  Label => MACRO,
}

impl Server {
  pub fn semantic_tokens(&self, uri: &Uri) -> SemanticTokens {
    let Some(doc) = self.get_document(uri) else {
      return SemanticTokens {
        result_id: None,
        data: vec![],
      };
    };

    let mut tokens = Vec::new();
    let mut prev_pos = Position::default();

    for token in collect_semantic_tokens(&doc.file.red_tree()) {
      let text = doc.file.text_of(token.range);
      let pos = doc.offset_to_lsp_position(token.range.start());

      let start_character = if pos.line == prev_pos.line {
        prev_pos.character
      } else {
        0
      };

      tokens.push(LspSemanticToken {
        delta_line: pos.line - prev_pos.line,
        delta_start: pos.character - start_character,
        length: WideEncoding::Utf16.measure(text) as u32,
        token_type: token.token_type as u32,
        token_modifiers_bitset: 0,
      });

      prev_pos = pos;
    }

    SemanticTokens {
      result_id: None,
      data: tokens,
    }
  }
}
