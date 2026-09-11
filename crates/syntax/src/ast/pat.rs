use crate::ast::Name;

define_node_enum! {
  Pattern {
    Wildcard(WildcardPat),
    Ident(IdentPat),
    Tuple(TuplePat),
    Record(RecordPat),
    Error(ErrorPat),
  }
}

define_nodes! {
  WildcardPat: _,
  IdentPat: _,
  TuplePat: _,
  RecordPat: _,
  PatField: _,
  ErrorPat: _,
}

impl IdentPat {
  define_getter! {
    mutable => ? KwMut;
    name => #0 Name;
  }
}

impl TuplePat {
  define_getter! {
    fields => [Pattern];
  }
}

impl RecordPat {
  define_getter! {
    fields => [PatField];
  }
}

impl PatField {
  define_getter! {
    // `None` for positional tuple elements, which have no name.
    name => #0 Name;
    pat => #-1 Pattern;
  }
}
