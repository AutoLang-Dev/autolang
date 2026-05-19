use crate::ast::{Expr, Name, Path};
use parser::SyntaxKind;

define_node_enum! {
  Type {
    Infer(InferType),
    Path(PathType),
    FnPtr(FnPtrType),
    Ref(RefType),
    Ptr(PtrType),
    Tuple(TupleType),
    Paren(ParenType),
    Array(ArrayType),
    Slice(SliceType),
    Struct(StructType),
    Error(ErrorType),
  }
}

define_nodes! {
  TupleField: _,
  StructField: _,
}

define_attr_vis! {
  TupleField,
  StructField,
}

impl TupleField {
  define_getter! {
    ty => ! Type;
  }
}

impl StructField {
  pub fn name(&self) -> Name {
    self
      .red
      .children()
      .find(|child| child.kind() == SyntaxKind::FieldName)
      .and_then(|x| Name::new(x.first_token()))
      .unwrap()
  }

  define_getter! {
    ty => ! Type;
  }
}

define_nodes! {
  InferType: _,
  PathType: _,
  FnPtrType: _,
  RefType: _,
  PtrType: _,
  TupleType: _,
  ParenType: _,
  ArrayType: _,
  SliceType: _,
  StructType: _,
  ErrorType: _,
}

impl PathType {
  define_getter! {
    path => ! Path;
  }
}

impl FnPtrType {
  define_getter! {
    mutable => ? KwMut;
    params => ! TupleType;
    ret => ! Type;
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
    fileds => [TupleField];
  }
}

impl ParenType {
  define_getter! {
    ty => ! Type;
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

impl StructType {
  define_getter! {
    fields => [StructField];
  }
}
