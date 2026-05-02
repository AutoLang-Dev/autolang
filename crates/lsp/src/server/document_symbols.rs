use crate::server::{Document, Server};
use lsp_types::{DocumentSymbol, SymbolKind, Uri};
use syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

impl Server {
  pub fn document_symbols(&self, uri: &Uri) -> Option<Vec<DocumentSymbol>> {
    self
      .document(uri)
      .map(|document| document.document_symbols())
  }
}

impl Document {
  fn document_symbols(&self) -> Vec<DocumentSymbol> {
    let root = self.parse.syntax_node();
    let Some(module_inner) = root
      .children()
      .find(|node| node.kind() == SyntaxKind::ModuleInner)
    else {
      return Vec::new();
    };

    self.module_symbols(&module_inner)
  }

  fn module_symbols(&self, module_inner: &SyntaxNode) -> Vec<DocumentSymbol> {
    module_inner
      .children()
      .filter_map(|node| self.item_symbol(&node))
      .collect()
  }

  fn item_symbol(&self, item: &SyntaxNode) -> Option<DocumentSymbol> {
    match item.kind() {
      SyntaxKind::ModuleItem => self.module_symbol(item),
      SyntaxKind::FunctionItem => self.named_symbol(item, SymbolKind::FUNCTION, None),
      SyntaxKind::TypeItem => self.type_symbol(item),
      SyntaxKind::BindingItem => self.binding_symbol(item),
      _ => None,
    }
  }

  fn module_symbol(&self, item: &SyntaxNode) -> Option<DocumentSymbol> {
    let children = item
      .children()
      .find(|node| node.kind() == SyntaxKind::ModuleInner)
      .map(|module_inner| self.module_symbols(&module_inner));

    self.named_symbol(item, SymbolKind::MODULE, children)
  }

  fn type_symbol(&self, item: &SyntaxNode) -> Option<DocumentSymbol> {
    let children = item
      .children()
      .find(|node| node.kind() == SyntaxKind::StructType)
      .map(|struct_type| self.struct_field_symbols(&struct_type));

    self.named_symbol(item, SymbolKind::STRUCT, children)
  }

  fn binding_symbol(&self, item: &SyntaxNode) -> Option<DocumentSymbol> {
    let name = item
      .children()
      .find(|node| node.kind() == SyntaxKind::Binding)
      .and_then(|binding| first_token_kind(&binding, is_ident_or_underscore))?;

    Some(self.symbol_from_name(item, name, SymbolKind::VARIABLE, None))
  }

  fn struct_field_symbols(&self, struct_type: &SyntaxNode) -> Vec<DocumentSymbol> {
    struct_type
      .children()
      .filter(|node| node.kind() == SyntaxKind::StructField)
      .filter_map(|field| self.struct_field_symbol(&field))
      .collect()
  }

  fn struct_field_symbol(&self, field: &SyntaxNode) -> Option<DocumentSymbol> {
    let name = field
      .children()
      .find(|node| node.kind() == SyntaxKind::FieldName)
      .and_then(|field_name| first_token_kind(&field_name, is_field_name_token))?;

    Some(self.symbol_from_name(field, name, SymbolKind::FIELD, None))
  }

  fn named_symbol(
    &self,
    node: &SyntaxNode,
    kind: SymbolKind,
    children: Option<Vec<DocumentSymbol>>,
  ) -> Option<DocumentSymbol> {
    let name = name_before_colon(node)?;
    Some(self.symbol_from_name(node, name, kind, children))
  }

  fn symbol_from_name(
    &self,
    node: &SyntaxNode,
    name: SyntaxToken,
    kind: SymbolKind,
    children: Option<Vec<DocumentSymbol>>,
  ) -> DocumentSymbol {
    #[allow(deprecated)]
    DocumentSymbol {
      name: name.text().to_string(),
      detail: None,
      kind,
      tags: None,
      deprecated: None,
      range: self.text_range_to_lsp(node.text_range()),
      selection_range: self.text_range_to_lsp(name.text_range()),
      children: non_empty(children),
    }
  }
}

fn name_before_colon(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .children_with_tokens()
    .filter_map(|element| element.into_token())
    .take_while(|token| token.kind() != SyntaxKind::Colon)
    .find(|token| is_ident_or_underscore(token.kind()))
}

fn first_token_kind(
  node: &SyntaxNode,
  predicate: impl Fn(SyntaxKind) -> bool,
) -> Option<SyntaxToken> {
  node
    .descendants_with_tokens()
    .filter_map(|element| element.into_token())
    .find(|token| predicate(token.kind()))
}

fn is_ident_or_underscore(kind: SyntaxKind) -> bool {
  matches!(kind, SyntaxKind::Ident | SyntaxKind::Underscore)
}

fn is_field_name_token(kind: SyntaxKind) -> bool {
  matches!(kind, SyntaxKind::Ident | SyntaxKind::Int)
}

fn non_empty(children: Option<Vec<DocumentSymbol>>) -> Option<Vec<DocumentSymbol>> {
  children.filter(|children| !children.is_empty())
}
