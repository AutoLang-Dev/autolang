pub trait Node {
  fn red(&self) -> &Red;

  fn green(&self) -> &Green {
    self.red().green()
  }
}

macro_rules! define_nodes {
  ($($name:ident $(: $kind:tt)?),+ $(,)?) => {
    $(
      pub struct $name {
        red: $crate::Red,
      }

      impl $name {
        define_nodes!(@new $name $($kind)?);
      }

      impl $crate::ast::Node for $name {
        fn red(&self) -> &$crate::Red {
          &self.red
        }
      }
    )*
  };

  (@new $name:ident $kind:ident) => {
    pub const KIND: ::parser::SyntaxKind = ::parser::SyntaxKind::$kind;

    pub fn new(red: $crate::Red) -> Option<Self> {
      if red.kind() == Self::KIND {
        Some(Self { red })
      } else {
        None
      }
    }
  };

  (@new $name:ident _) => {
    define_nodes!(@new $name $name);
  };

  (@new $name:ident) => {
    pub fn new(red: $crate::Red) -> Self {
      Self { red }
    }
  };
}

/// Like [`define_nodes!`], for types that wrap a single token.
///
/// These types are found among a node's token children, their `KIND` is a token
/// kind, and they expose the token text.
macro_rules! define_token_nodes {
  ($($name:ident $(: $kind:tt)?),+ $(,)?) => {
    $(
      pub struct $name {
        red: $crate::Red,
      }

      impl $name {
        define_token_nodes!(@new $name $($kind)?);
      }

      impl $crate::ast::Node for $name {
        fn red(&self) -> &$crate::Red {
          &self.red
        }
      }
    )*
  };

  (@new $name:ident $kind:ident) => {
    pub const KIND: ::parser::SyntaxKind = ::parser::SyntaxKind::$kind;

    pub fn new(red: $crate::Red) -> Option<Self> {
      if red.kind() == Self::KIND && red.is_token() {
        Some(Self { red })
      } else {
        None
      }
    }
  };

  (@new $name:ident _) => {
    define_token_nodes!(@new $name $name);
  };
}

macro_rules! define_node_enum {
  ($name:ident {$($var:ident($inner:ty)),+ $(,)?} $($flag:tt)?) => {
    pub enum $name {
      $($var($inner),)*
    }

    impl $name {
      define_node_enum!(@new {$($var($inner),)*} $($flag)?);
    }

    impl $crate::ast::Node for $name {
      fn red(&self) -> &$crate::Red {
        match self {
          $(Self::$var(p) => p.red(),)*
        }
      }
    }
  };

  (@new {$($var:ident($inner:ty)),+ $(,)?}) => {
    pub fn new(red: $crate::Red) -> Option<Self> {
      Some(match red.kind() {
        $(
          <$inner>::KIND => Self::$var(<$inner>::new(red)?),
        )*
        _ => return None,
      })
    }
  };

  (@new $t:tt no_new) => {};
}

macro_rules! define_getter {
  ($f:ident => ? $k:ident; $($rest:tt)*) => {
    pub fn $f(&self) -> bool {
      use $crate::ast::Node;
      self.red().children()
      .any(|child| child.kind() == parser::SyntaxKind::$k)
    }
    define_getter!($($rest)*);
  };

  ($f:ident => [$r:ty]; $($rest:tt)*) => {
    pub fn $f(&self) -> Vec<$r> {
      use $crate::ast::Node;
      self.red().children().filter_map(<$r>::new).collect()
    }
    define_getter!($($rest)*);
  };

  ($f:ident => #-$n:literal $r:ty; $($rest:tt)*) => {
    pub fn $f(&self) -> Option<$r> {
      use $crate::ast::Node;
      $crate::ast::node_child_from_end(self.red(), $n)
      .and_then(<$r>::new)
    }
    define_getter!($($rest)*);
  };

  ($f:ident => #$n:literal $r:ty; $($rest:tt)*) => {
    pub fn $f(&self) -> Option<$r> {
      use $crate::ast::Node;
      $crate::ast::node_children(self.red()).nth($n).and_then(<$r>::new)
    }
    define_getter!($($rest)*);
  };

  ($f:ident => $r:ty; $($rest:tt)*) => {
    pub fn $f(&self) -> Option<$r> {
      use $crate::ast::Node;
      self.red().children().find_map(<$r>::new)
    }
    define_getter!($($rest)*);
  };

  () => {};
}

/// Non-token children, in source order.
pub(crate) fn node_children(red: &Red) -> impl DoubleEndedIterator<Item = Red> + '_ {
  red.children().filter(|child| !child.is_token())
}

/// The `n`-th non-token child from the end, one-based (`1` is the last).
pub(crate) fn node_child_from_end(red: &Red, n: usize) -> Option<Red> {
  node_children(red).rev().nth(n.checked_sub(1)?)
}

#[macro_use]
mod attrs;
mod expr;
mod items;
mod pat;
mod paths;
mod stmts;
mod types;

pub use {attrs::*, expr::*, items::*, pat::*, paths::*, stmts::*, types::*};

use {
  crate::{Green, Red},
  parser::SyntaxKind,
};

define_nodes! {
  Root: SourceFile,
}

impl Root {
  pub fn items(&self) -> Vec<Item> {
    items_in_file_or_module(&self.red)
  }
}

fn items_in_file_or_module(outer: &Red) -> Vec<Item> {
  let Some(inner) = outer
    .children()
    .find(|child| child.kind() == SyntaxKind::ModuleInner)
  else {
    return Vec::new();
  };

  inner.children().filter_map(Item::new).collect()
}

define_nodes! {
  TokenTree: TokenTree,
  DelimitedTokenTree: DelimitedTokenTree,
  Name: Name,
}
