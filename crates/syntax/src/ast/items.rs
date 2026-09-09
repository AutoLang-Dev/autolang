use crate::{
  Red,
  ast::{Expr, FnType, Name, Type, items_in_file_or_module},
};
use parser::SyntaxKind;

define_node_enum! {
  Item {
    Function(FunctionItem),
    Type(TypeItem),
    Using(UsingItem),
    Module(ModuleItem),
    Empty(EmptyItem),
    Error(ErrorItem),
  }
}

define_nodes! {
  FunctionItem: _,
  TypeItem: _,
  UsingItem: _,
  ModuleItem: _,
  EmptyItem: _,
  ErrorItem: _,
}

define_attr_vis! {
  Item,
  FunctionItem,
  TypeItem,
  UsingItem,
  ModuleItem,
  EmptyItem,
  ErrorItem,
}

impl FunctionItem {
  define_getter! {
    name => ! Name;
    ty => FnType;
    body <= Expr;
  }
}

define_node_enum! {
  TypeKind {
    Alias(AliasType),
    New(NewType),
  } no_new
}

define_nodes! {
  AliasType: KwType,
  NewType: KwNominal,
}

impl TypeKind {
  pub fn new(red: Red) -> Option<Self> {
    if red.kind() != SyntaxKind::TypeKind {
      return None;
    }

    let kind = red.first_token();
    Some(match kind.kind() {
      AliasType::KIND => Self::Alias(AliasType::new(kind)?),
      NewType::KIND => Self::New(NewType::new(kind)?),
      _ => return None,
    })
  }
}

impl TypeItem {
  define_getter! {
    name => ! Name;
    kind => ! TypeKind;
    ty <= ! Type;
  }
}

impl ModuleItem {
  define_getter! {
    name => ! Name;
  }

  pub fn items(&self) -> Option<Vec<Item>> {
    self
      .red
      .children()
      .find(|child| child.kind() == SyntaxKind::Module)
      .as_ref()
      .map(items_in_file_or_module)
  }
}
