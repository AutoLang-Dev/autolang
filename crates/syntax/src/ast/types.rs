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
    name => Name;
    ty => #-1 Type;
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
    path => #0 Path;
  }
}

impl FnType {
  define_getter! {
    mutable => ? KwMut;
    params => #0 Type;
    ret => #-1 Type;
  }
}

impl RefType {
  define_getter! {
    mutable => ? KwMut;
    pointee => #0 Type;
  }
}

impl PtrType {
  define_getter! {
    mutable => ? KwMut;
    pointee => #0 Type;
  }
}

impl TupleType {
  define_getter! {
    fields => [TypeField];
  }
}

impl ArrayType {
  define_getter! {
    ty => #0 Type;
    len => #-1 Expr;
  }
}

impl SliceType {
  define_getter! {
    ty => #0 Type;
  }
}

impl RecordType {
  define_getter! {
    fields => [TypeField];
  }
}
