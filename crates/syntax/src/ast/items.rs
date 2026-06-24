use crate::{
  Red,
  ast::{Expr, Name, Pattern, Type, items_in_file_or_module},
};
use parser::SyntaxKind;

define_node_enum! {
  Item {
    Binding(BindingItem),
    Function(FunctionItem),
    Type(TypeItem),
    Using(UsingItem),
    Module(ModuleItem),
    Empty(EmptyItem),
    Error(ErrorItem),
  }
}

define_nodes! {
  BindingItem: _,
  FunctionItem: _,
  TypeItem: _,
  UsingItem: _,
  ModuleItem: _,
  EmptyItem: _,
  ErrorItem: _,
}

define_attr_vis! {
  Item,
  BindingItem,
  FunctionItem,
  TypeItem,
  UsingItem,
  ModuleItem,
  EmptyItem,
  ErrorItem,
}

impl BindingItem {
  define_getter! {
    pat => ! Pattern;
    ty => Type;
    init <= Expr;
  }
}

define_nodes! {
  ParameterList: ParameterList,
  Parameter: Parameter,
}

impl ParameterList {
  define_getter! {
    params => [Parameter];
  }
}

impl Parameter {
  define_getter! {
    pat => ! Pattern;
    ty <= ! Type;
  }
}

impl FunctionItem {
  define_getter! {
    name => ! Name;
    params => ParameterList;
    mutable => ? KwMut;
    ret <= ! Name;
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
