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
  ($f:ident $dir:tt ! $r:ty; $($rest:tt)*) => {
    pub fn $f(&self) -> $r {
      use $crate::ast::Node;
      define_getter!(@dir $dir self.red().children())
      .find_map(<$r>::new).unwrap()
    }
    define_getter!($($rest)*);
  };

  ($f:ident $dir:tt ? $k:ident; $($rest:tt)*) => {
    pub fn $f(&self) -> bool {
      use $crate::ast::Node;
      define_getter!(@dir $dir self.red().children())
      .any(|child| child.kind() == parser::SyntaxKind::$k)
    }
    define_getter!($($rest)*);
  };

  ($f:ident $dir:tt [$r:ty]; $($rest:tt)*) => {
    pub fn $f(&self) -> Vec<$r> {
      use $crate::ast::Node;
      define_getter!(@dir $dir self.red().children())
      .filter_map(<$r>::new).collect()
    }
    define_getter!($($rest)*);
  };

  ($f:ident $dir:tt $r:ty; $($rest:tt)*) => {
    pub fn $f(&self) -> Option<$r> {
      use $crate::ast::Node;
      define_getter!(@dir $dir self.red().children())
      .find_map(<$r>::new)
    }
    define_getter!($($rest)*);
  };

  (@dir => $e:expr) => { $e };
  (@dir <= $e:expr) => { $e.rev() };

  () => {};
}

#[macro_use]
mod attrs;
mod expr;
mod items;
mod literals;
mod pat;
mod paths;
mod stmts;
mod types;

pub use {attrs::*, expr::*, items::*, pat::*, paths::*, stmts::*, types::*};

use {
  crate::{Green, Red},
  parser::{SyntaxKind, T},
};

define_nodes! {
  Shebang: Shebang,
  Root: SourceFile,
}

impl Root {
  define_getter! {
    shebang => Shebang;
  }

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
}

pub struct Name {
  red: Red,
}

impl Name {
  pub fn new(red: Red) -> Option<Self> {
    if !matches!(
      red.kind(),
      SyntaxKind::Ident | SyntaxKind::Int | SyntaxKind::Error | T![_]
    ) {
      return None;
    }
    Some(Self { red })
  }
}

impl Node for Name {
  fn red(&self) -> &Red {
    &self.red
  }
}
