//! Minimal red nodes for the contract tests.

use crate::Green;
use crate::Red;
use crate::SyntaxKind;
use crate::SyntaxKind::*;
use crate::make::Frag;

/// A red node for `kind` with the children its AST type needs to cast.
pub(super) fn probe_red(kind: SyntaxKind) -> Red {
  match kind {
    // These enums dispatch on a token inside the node.
    TypeKind => Frag::node(kind, [Frag::kind_token(KwType)]).red(),
    Visibility => Frag::node(kind, [Frag::kind_token(KwPub)]).red(),
    _ => Red::new_root(Green::node(kind, [])),
  }
}
