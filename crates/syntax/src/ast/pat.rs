use crate::ast::Name;

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
  TuplePat: _,
  RecordPat: _,
  PatField: _,
  ErrorPat: _,
}

impl IdentPat {
  define_getter! {
    mutable => ? KwMut;
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
    name => ! Name;
    pat <= Pattern;
  }
}
