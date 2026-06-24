use crate::server::Server;
use line_index::WideEncoding;
use lsp_types::{
  Position, SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, Uri,
};
use parser::{SyntaxKind, T};
use syntax::Red;

macro_rules! define_tokens {
  ( $($name:ident => $token:ident),* $(,)? ) => {
    #[repr(u32)]
    enum TokenType {
      $(
        $name,
      )*
    }
    use TokenType::*;

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

fn map_name(name: &Red) -> Option<TokenType> {
  use SyntaxKind as S;

  let parent = name.parent()?;
  let ty = match parent.kind() {
    S::Rename => Module,
    S::ModuleItem => Module,
    S::TypeItem => Type,
    S::FunctionItem => Function,
    S::FieldName | S::FieldExpr => Field,
    S::PathSegment => 'blk: {
      let path = parent.parent()?;
      if path.last_child().unwrap().green() != parent.green() {
        break 'blk Module;
      }

      let parent = path.parent()?;
      match parent.kind() {
        S::AttrItem => Decorator,
        S::MethodCallExpr => Method,
        S::PathType => Type,
        S::UsingTree => Module,
        S::PathExpr => {
          if let Some(parent) = parent.parent()
            && parent.kind() == S::CallExpr
          {
            Function
          } else {
            Ident
          }
        }
        _ => unreachable!(),
      }
    }
    _ => return None,
  };

  Some(ty)
}

fn map_token(token: &Red) -> Option<TokenType> {
  use SyntaxKind as S;

  let ty = match token.kind() {
    S::Ident | T![_] => map_name(token).unwrap_or(Ident),
    S::Int => Number,
    S::Char | S::Byte | S::String | S::RawString => String,
    S::Comment | S::Shebang => Comment,
    S::Label => Label,
    T![pub] | T![pro] | T![pri] => Modifier,
    s if s.is_keyword() => Keyword,
    _ => return None,
  };

  Some(ty)
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

    for token in doc.syntax_tree().tokens() {
      let text = doc.text_of(token.range());

      let Some(ty) = map_token(&token) else {
        continue;
      };

      let pos = doc.offset_to_lsp_position(token.offset());

      let start_character = if pos.line == prev_pos.line {
        prev_pos.character
      } else {
        0
      };

      tokens.push(SemanticToken {
        delta_line: pos.line - prev_pos.line,
        delta_start: pos.character - start_character,
        length: WideEncoding::Utf16.measure(text) as u32,
        token_type: ty as u32,
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
