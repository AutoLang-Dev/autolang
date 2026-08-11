use parser::{SyntaxKind, T};
use syntax::Red;
use text_size::TextRange;

pub enum TokenType {
  Comment,
  Keyword,
  Module,
  Function,
  Type,
  Field,
  Method,
  Decorator,
  Modifier,
  Ident,
  String,
  Number,
  Label,
}

use TokenType::*;

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

pub struct SemanticToken {
  pub range: TextRange,
  pub token_type: TokenType,
}

pub fn collect_semantic_tokens(root: &Red) -> Vec<SemanticToken> {
  root
    .tokens()
    .filter_map(|token| {
      let ty = map_token(&token)?;
      Some(SemanticToken {
        range: token.range(),
        token_type: ty,
      })
    })
    .collect()
}
