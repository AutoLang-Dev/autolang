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
    Array(ArrayType),
    Slice(SliceType),
    Record(RecordType),
    Error(ErrorType),
  }
}

define_nodes! {
  TupleField: _,
  TypeField: _,
}

define_attr_vis! {
  TupleField,
  TypeField,
}

impl TupleField {
  define_getter! {
    ty => ! Type;
  }
}

impl TypeField {
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
