//! Programmatic construction of syntax tree fragments.
//!
//! A [`Frag`] pairs a green fragment with the exact source text it was built
//! from, so trees can be assembled without a lexer or parser while callers can
//! still slice text out of the paired `String` by range.
//!
//! The [`frag!`](crate::frag) macro is the concise form. It writes one node or
//! token per item, and `@(expr)` splices in anything that yields fragments,
//! which is how lists built at runtime get into a static shape:
//!
//! ```ignore
//! let field = |name: &str| frag! {
//!   TypeField { Name { Ident(name) } Colon PathType { Path { PathSegment { Name { Ident("Int") } } } } }
//! };
//! let fields = make::joined(Comma, ["a", "b"].map(field));
//!
//! let (green, src) = frag! { TupleType { OpenParen @(fields) CloseParen } }.finish();
//! ```
//!
//! There are deliberately no constructors for individual node kinds: they would
//! be a second copy of the grammar, to be kept in sync by hand. The shape of a
//! tree is written where it is used, and [`joined`] covers the one thing the
//! macro cannot express — a list whose length is only known at runtime.

use crate::{DiagPayload, Green, Payload, Red};
use parser::{Error, SyntaxKind};
use std::{iter::Once, string::String, vec::Vec};
use text_size::TextSize;

/// A green fragment plus the source text it was built from.
///
/// The text is accumulated as the fragment is assembled, so it can never drift
/// from the tree's widths.
#[derive(Debug, Clone)]
pub struct Frag {
  green: Green,
  text: String,
}

impl Frag {
  /// A token whose width is derived from `text`.
  pub fn token(kind: SyntaxKind, text: &str) -> Self {
    Self {
      green: Green::token(kind, TextSize::of(text), Payload::default()),
      text: text.to_owned(),
    }
  }

  /// A token that spells itself, e.g. `Colon` or `ThinArrow`.
  ///
  /// Panics for kinds without fixed text (`Ident`, literals, `Label`); use
  /// [`Frag::token`] for those.
  pub fn kind_token(kind: SyntaxKind) -> Self {
    Self::token(
      kind,
      kind
        .fixed_text()
        .expect("kind has no fixed text; use Frag::token"),
    )
  }

  /// A zero-width token, for malformed or error shapes.
  pub fn empty(kind: SyntaxKind) -> Self {
    Self {
      green: Green::token(kind, TextSize::new(0), Payload::default()),
      text: String::new(),
    }
  }

  /// A zero-width token carrying a diagnostic.
  pub fn diag_token(kind: SyntaxKind, text: &str, error: Error) -> Self {
    Self {
      green: Green::token(
        kind,
        TextSize::of(text),
        Payload {
          diag: Some(DiagPayload::Diag(error)),
        },
      ),
      text: text.to_owned(),
    }
  }

  /// A node over `parts`, in source order.
  pub fn node(kind: SyntaxKind, parts: impl IntoIterator<Item = Frag>) -> Self {
    let mut greens = Vec::new();
    let mut text = String::new();

    for part in parts {
      greens.push(part.green);
      text.push_str(&part.text);
    }

    Self {
      green: Green::node(kind, greens),
      text,
    }
  }

  /// The fragment's source text.
  pub fn text(&self) -> &str {
    &self.text
  }

  /// Splits into the green fragment and its source text.
  pub fn finish(self) -> (Green, String) {
    (self.green, self.text)
  }

  /// Views the fragment as a red root. Any kind may be a root.
  pub fn red(self) -> Red {
    Red::new_root(self.green)
  }
}

/// A single fragment splices as itself, so `@x` works for `x: Frag` and for
/// any list of fragments alike.
impl IntoIterator for Frag {
  type Item = Frag;
  type IntoIter = Once<Frag>;

  fn into_iter(self) -> Self::IntoIter {
    std::iter::once(self)
  }
}

/// `items` joined by `sep`, matching how the parser separates lists.
pub fn joined(sep: SyntaxKind, items: impl IntoIterator<Item = Frag>) -> Vec<Frag> {
  let mut parts = Vec::new();

  for (index, item) in items.into_iter().enumerate() {
    if index > 0 {
      parts.push(Frag::kind_token(sep));
    }

    parts.push(item);
  }

  parts
}

/// Builds a fragment tree, one item per node or token.
///
/// * `Kind { .. }` is a node,
/// * `Kind` is a token that spells itself,
/// * `Kind("text")` is a token with that text,
/// * `@expr` splices in whatever `expr` yields, e.g. [`joined`] output.
#[macro_export]
macro_rules! frag {
  ($kind:ident { $($child:tt)* }) => {
    $crate::make::Frag::node(
      $crate::SyntaxKind::$kind,
      $crate::frag!(@parts ::std::iter::empty() ; $($child)*),
    )
  };

  ($kind:ident ( $text:expr )) => {
    $crate::make::Frag::token($crate::SyntaxKind::$kind, $text)
  };

  ($kind:ident) => {
    $crate::make::Frag::kind_token($crate::SyntaxKind::$kind)
  };

  (@parts $parts:expr ; ) => {
    $parts
  };

  (@parts $parts:expr ; @ ( $splice:expr ) $($rest:tt)*) => {
    $crate::frag!(@parts ::std::iter::Iterator::chain($parts, $splice) ; $($rest)*)
  };

  (@parts $parts:expr ; $kind:ident { $($inner:tt)* } $($rest:tt)*) => {
    $crate::frag!(@parts ::std::iter::Iterator::chain(
      $parts,
      [$crate::frag!($kind { $($inner)* })],
    ) ; $($rest)*)
  };

  (@parts $parts:expr ; $kind:ident ( $text:expr ) $($rest:tt)*) => {
    $crate::frag!(@parts ::std::iter::Iterator::chain(
      $parts,
      [$crate::frag!($kind ( $text ))],
    ) ; $($rest)*)
  };

  (@parts $parts:expr ; $kind:ident $($rest:tt)*) => {
    $crate::frag!(@parts ::std::iter::Iterator::chain(
      $parts,
      [$crate::frag!($kind)],
    ) ; $($rest)*)
  };
}
