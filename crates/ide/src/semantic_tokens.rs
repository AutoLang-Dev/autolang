use crate::FileId;
use async_inc::Query;
use line_index::TextRange;
use parser::{SyntaxKind, T};
use syntax::Red;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

  // Names live in a `Name` node, so the token's parent is the wrapper and the
  // wrapper's parent is what decides the role. Error recovery can leave stray
  // identifiers outside of any `Name`, hence the fallback.
  let parent = name.parent()?;
  let parent = if parent.kind() == S::Name {
    parent.parent()?
  } else {
    parent
  };
  let ty = match parent.kind() {
    S::Rename => Module,
    S::ModuleItem => Module,
    S::TypeItem => Type,
    S::FunctionItem => Function,
    S::TypeField | S::ExprField | S::FieldExpr | S::PatField => Field,
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
    S::Comment => Comment,
    S::Label => Label,
    T![pub] | T![pro] | T![pri] => Modifier,
    s if s.is_keyword() => Keyword,
    _ => return None,
  };

  Some(ty)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticToken {
  pub range: TextRange,
  pub token_type: TokenType,
}

fn collect_semantic_tokens(root: &Red) -> Vec<SemanticToken> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GetTokens(pub FileId);

impl Query for GetTokens {
  type Value = Vec<SemanticToken>;

  async fn compute(self, ctx: async_inc::QueryCtx) -> Self::Value {
    let file = ctx.get(self.0);
    collect_semantic_tokens(&file.red_tree())
  }
}
