use crate::server::Server;
use lexer::{TokenKind, lex, strip_shebang};
use line_index::{TextSize, WideEncoding};
use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, Uri};

macro_rules! define_tokens {
  ( $($name:ident => $token:ident),* $(,)? ) => {
    #[repr(u32)]
    enum TokenType {
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

macro_rules! define_keywords {
  ($($kw:ident),* $(,)?) => {
    fn is_keyword(text: &str) -> bool {
      matches!(text, $(stringify!($kw))|*)
    }
  };
}

define_tokens! {
  Comment => COMMENT,
  Keyword => KEYWORD,
  Ident => VARIABLE,
  String => STRING,
  Number => NUMBER,
  Operator => OPERATOR,
  Label => MACRO,
}

define_keywords!(
  as, break, case, cont, else, fn, for, if, impl, iterate, mod, mut, nominal, pri, pro, pub,
  return, self, type, using, while, true, false
);

fn map_token_kind(kind: TokenKind, text: &str) -> Option<u32> {
  if kind == TokenKind::Whitespace {
    return None;
  }

  let tt = match kind {
    TokenKind::Comment => TokenType::Comment,
    TokenKind::Literal => {
      let bytes = text.as_bytes();
      match bytes[0] {
        b'"' | b'\'' | b'b' => TokenType::String,
        b'0'..=b'9' => TokenType::Number,
        _ => unreachable!(),
      }
    }
    TokenKind::Ident => {
      if is_keyword(text) {
        TokenType::Keyword
      } else {
        TokenType::Ident
      }
    }
    TokenKind::Label => TokenType::Label,
    _ => TokenType::Operator,
  };
  Some(tt as u32)
}

impl Server {
  pub fn semantic_tokens(&self, uri: &Uri) -> SemanticTokens {
    let Some((src, index)) = self.get_document(uri) else {
      return SemanticTokens {
        result_id: None,
        data: vec![],
      };
    };

    let shebang_len = strip_shebang(src);

    let mut tokens = Vec::new();
    let mut pos = TextSize::new(shebang_len as u32);
    let (shebang, mut rest) = src.split_at(shebang_len);
    let (mut prev_line, mut prev_col) = (0, 0);

    if shebang_len != 0 {
      tokens.push(SemanticToken {
        delta_line: 0,
        delta_start: 0,
        length: WideEncoding::Utf16.measure(shebang) as u32,
        token_type: TokenType::Comment as u32,
        token_modifiers_bitset: 0,
      });
    }

    for token in lex(rest) {
      let text;
      (text, rest) = rest.split_at(token.len as usize);

      let Some(token_type) = map_token_kind(token.kind, text) else {
        pos += TextSize::new(token.len);
        continue;
      };

      let line_col = index.line_col(pos);
      let line_col = index.to_wide(WideEncoding::Utf16, line_col).unwrap();

      let start_col = if line_col.line == prev_line {
        prev_col
      } else {
        0
      };

      tokens.push(SemanticToken {
        delta_line: line_col.line - prev_line,
        delta_start: line_col.col - start_col,
        length: WideEncoding::Utf16.measure(text) as u32,
        token_type,
        token_modifiers_bitset: 0,
      });

      (prev_line, prev_col) = (line_col.line, line_col.col);
      pos += TextSize::new(token.len);
    }

    SemanticTokens {
      result_id: None,
      data: tokens,
    }
  }
}
