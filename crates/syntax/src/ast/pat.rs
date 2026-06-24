define_node_enum! {
  Pattern {
    Wildcard(WildcardPat),
    Ident(IdentPat),
    Error(ErrorPat),
  }
}

define_nodes! {
  WildcardPat: _,
  IdentPat: _,
  ErrorPat: _,
}

impl IdentPat {
  define_getter! {
    mutable => ? KwMut;
  }
}
