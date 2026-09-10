use crate::ast::{Expr, Name, Path};

define_node_enum! {
  Type {
    Infer(InferType),
    Path(PathType),
    Fn(FnType),
    Ref(RefType),
    Ptr(PtrType),
    Tuple(TupleType),
    Array(ArrayType),
    Slice(SliceType),
    Record(RecordType),
    Error(ErrorType),
  }
}

define_nodes! {
  TypeField: _,
}

define_attr_vis! {
  TypeField,
}

impl TypeField {
  define_getter! {
    // `None` for positional tuple type elements, which have no name.
    name => Name;
    ty => ! Type;
  }
}

define_nodes! {
  InferType: _,
  PathType: _,
  FnType: _,
  RefType: _,
  PtrType: _,
  TupleType: _,
  ArrayType: _,
  SliceType: _,
  RecordType: _,
  ErrorType: _,
}

impl PathType {
  define_getter! {
    path => ! Path;
  }
}

impl FnType {
  define_getter! {
    mutable => ? KwMut;
    params => ! Type;
    ret <= ! Type;
  }
}

impl RefType {
  define_getter! {
    mutable => ? KwMut;
    pointee => ! Type;
  }
}

impl PtrType {
  define_getter! {
    mutable => ? KwMut;
    pointee => ! Type;
  }
}

impl TupleType {
  define_getter! {
    fields => [TypeField];
  }
}

impl ArrayType {
  define_getter! {
    ty => ! Type;
    len <= ! Expr;
  }
}

impl SliceType {
  define_getter! {
    ty => ! Type;
  }
}

impl RecordType {
  define_getter! {
    fields => [TypeField];
  }
}
