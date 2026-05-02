use crate::SyntaxKind;
use rowan::Language;

pub type SyntaxNode = rowan::SyntaxNode<Lang>;
pub type SyntaxToken = rowan::SyntaxToken<Lang>;
pub type SyntaxElement = rowan::SyntaxElement<Lang>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}

impl Language for Lang {
  type Kind = SyntaxKind;

  fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
    SyntaxKind::from_raw(raw.0)
  }

  fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
    rowan::SyntaxKind(kind.into_raw())
  }
}
