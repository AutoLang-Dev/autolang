use crate::ast::Name;
use parser::SyntaxKind;

define_nodes! {
  Path: _,
  PathSegment: _,
}

impl Path {
  define_getter! {
    segments => [PathSegment];
  }

  /// Whether the path was written with a leading `::`.
  ///
  /// The two forms differ in what their first segment names:
  ///
  /// ```text
  /// ::a::b   the unit `a`, then the module `b`
  /// a::b     the module `b` inside `a`, from where the path is written
  /// ```
  pub fn global(&self) -> bool {
    // Not `children().next()`: the first child of `a::b` is a `PathSegment`,
    // and only a `::` written *before* the first segment counts.
    self.red.first_token().kind() == SyntaxKind::ColonColon
  }
}

impl PathSegment {
  /// The segment's shape, since the same node kind covers names and the three
  /// path keywords.
  pub fn kind(&self) -> PathSegmentKind {
    if let Some(name) = self.red.children().find_map(Name::new) {
      return PathSegmentKind::Name(name);
    }

    match self.red.first_token().kind() {
      SyntaxKind::KwSelf => PathSegmentKind::Self_,
      SyntaxKind::KwSuper => PathSegmentKind::Super,
      SyntaxKind::KwUnit => PathSegmentKind::Unit,
      _ => PathSegmentKind::Error,
    }
  }
}

/// The shape a [`PathSegment`] has.
///
/// `self`, `super` and `unit` are keywords, not names, so they are not wrapped
/// in a [`Name`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegmentKind {
  /// `foo`
  Name(Name),
  /// `self`
  Self_,
  /// `super`
  Super,
  /// `unit`
  Unit,
  /// Nothing usable, such as a missing segment.
  Error,
}

define_nodes! {
  UsingTree: _,
  UsingTreeList: _,
  UsingUnitName: _,
  Rename: _,
}

impl UsingTree {
  pub fn kind(&self) -> Option<UsingTreeKind> {
    let Some(first) = self.red.children().next() else {
      return None;
    };

    match first.kind() {
      SyntaxKind::Underscore => Some(UsingTreeKind::Glob),
      SyntaxKind::UsingTreeList => Some(UsingTreeKind::List {
        list: UsingTreeList::new(first)?,
      }),
      SyntaxKind::Name => {
        let name = Name::new(first).unwrap();
        if let Some(tree) = self.red.children().find_map(UsingTree::new) {
          Some(UsingTreeKind::Path { name, tree })
        } else {
          let rename = self.red.children().find_map(Rename::new);
          Some(UsingTreeKind::Leaf { name, rename })
        }
      }
      _ => None,
    }
  }
}

#[derive(Debug, Clone)]
pub enum UsingTreeKind {
  Leaf { name: Name, rename: Option<Rename> },
  Path { name: Name, tree: UsingTree },
  Glob,
  List { list: UsingTreeList },
}

impl UsingTreeList {
  define_getter! {
    trees => [UsingTree];
  }
}

impl Rename {
  define_getter! {
    name => Name;
  }
}
