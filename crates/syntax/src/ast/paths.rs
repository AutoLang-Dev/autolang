define_nodes! {
  Path: _,
  PathSegment: _,
}

define_node_enum! {
  Using {
    Tree(UsingTree),
    List(UsingTreeList),
  }
}

define_nodes! {
  UsingTree: _,
  UsingTreeList: _,
  Rename: _,
}
