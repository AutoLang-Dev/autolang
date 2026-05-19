use crate::{
  Red,
  ast::{DelimitedTokenTree, Expr, Path},
};
use parser::SyntaxKind;

macro_rules! define_attr {
  ($($name:ident),+ $(,)?) => {
    $(
      impl $name {
        define_getter! {
          attr => $crate::ast::Attr;
        }
      }
    )*
  };
}

macro_rules! define_vis {
  ($($name:ident),+ $(,)?) => {
    $(
      impl $name {
        define_getter! {
          vis => $crate::ast::Visibility;
        }
      }
    )*
  };
}

macro_rules! define_attr_vis {
  ($($t:tt)*) => {
    define_attr!{$($t)*}
    define_vis!{$($t)*}
  };
}

define_nodes! {
  Attr: _,
  AttrItem: _,
}

impl Attr {
  pub fn items(&self) -> Vec<AttrItem> {
    let Some(inner) = self.inner() else {
      return Vec::new();
    };

    inner.children().filter_map(AttrItem::new).collect()
  }

  fn inner(&self) -> Option<Red> {
    self
      .red
      .children()
      .find(|child| child.kind() == SyntaxKind::AttrInner)
  }
}

impl AttrItem {
  define_getter! {
    attr => ! Path;
    arg => AttrArg;
  }
}

define_node_enum! {
  AttrArg {
    Expr(Expr),
    Tree(DelimitedTokenTree),
  } no_new
}

impl AttrArg {
  pub fn new(red: Red) -> Option<Self> {
    Some(match red.kind() {
      DelimitedTokenTree::KIND => Self::Tree(DelimitedTokenTree::new(red)?),
      _ => Self::Expr(Expr::new(red)?),
    })
  }
}

define_node_enum! {
  Visibility {
    Pub(Pub),
    Pro(Pro),
    Pri(Pri),
  } no_new
}

impl Visibility {
  pub fn new(red: Red) -> Option<Self> {
    if red.kind() != SyntaxKind::Visibility {
      return None;
    }

    let vis = red.first_token();
    Some(match vis.kind() {
      Pub::KIND => Self::Pub(Pub::new(vis)?),
      Pro::KIND => Self::Pro(Pro::new(vis)?),
      Pri::KIND => Self::Pri(Pri::new(vis)?),
      _ => return None,
    })
  }
}

define_nodes! {
  Pub: KwPub,
  Pro: KwPro,
  Pri: KwPri,
}
