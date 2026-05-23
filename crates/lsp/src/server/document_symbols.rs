use crate::server::{Server, document::Document};
use lsp_types::{DocumentSymbol, SymbolKind, Uri};
use syntax::ast::*;

pub fn all_symbols(doc: &Document, root: Root) -> Vec<DocumentSymbol> {
  root
    .items()
    .into_iter()
    .filter_map(|item| item_symbol(doc, item))
    .collect()
}

fn item_symbol(doc: &Document, item: Item) -> Option<DocumentSymbol> {
  Some(match item {
    Item::Module(m) => module_symbol(doc, m),
    Item::Function(f) => funciton_symbol(doc, f),
    Item::Type(t) => type_symbol(doc, t),
    _ => return None,
  })
}

fn module_symbol(doc: &Document, m: ModuleItem) -> DocumentSymbol {
  let children = m
    .items()
    .map(|i| i.into_iter().filter_map(|x| item_symbol(doc, x)).collect());

  symbol(doc, &m, m.name(), SymbolKind::MODULE, children)
}

fn funciton_symbol(doc: &Document, f: FunctionItem) -> DocumentSymbol {
  symbol(doc, &f, f.name(), SymbolKind::FUNCTION, None)
}

fn type_symbol(doc: &Document, t: TypeItem) -> DocumentSymbol {
  let children = if let Type::Struct(s) = t.ty() {
    Some(field_symbols(doc, s))
  } else {
    None
  };

  symbol(doc, &t, t.name(), SymbolKind::STRUCT, children)
}

fn field_symbols(doc: &Document, s: StructType) -> Vec<DocumentSymbol> {
  s.fields()
    .iter()
    .map(|x| symbol(doc, x, x.name(), SymbolKind::FIELD, None))
    .collect()
}

fn symbol(
  doc: &Document,
  node: &impl Node,
  name: Name,
  kind: SymbolKind,
  children: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
  let node = node.red().range();
  let name = name.red().range();

  #[allow(deprecated)]
  DocumentSymbol {
    name: doc.text_of(name).to_string(),
    detail: None,
    kind,
    tags: None,
    deprecated: None,
    range: doc.span_to_lsp_range(node),
    selection_range: doc.span_to_lsp_range(name),
    children,
  }
}

impl Document {
  pub fn document_symbols(&self) -> Vec<DocumentSymbol> {
    let red = self.syntax_tree().clone();
    let root = Root::new(red).unwrap();
    all_symbols(self, root)
  }
}

impl Server {
  pub fn document_symbols(&self, uri: &Uri) -> Option<Vec<DocumentSymbol>> {
    self.get_document(uri).map(|doc| doc.document_symbols())
  }
}
