use crate::server::{Server, document::Document};
use ide::{DocumentSymbol, SymbolKind, collect_symbols};
use lsp_types::{DocumentSymbol as LspDocumentSymbol, SymbolKind as LspSymbolKind, Uri};
use syntax::ast::*;

fn map_symbol_kind(kind: SymbolKind) -> LspSymbolKind {
  match kind {
    SymbolKind::Module => LspSymbolKind::MODULE,
    SymbolKind::Type => LspSymbolKind::STRUCT,
    SymbolKind::Function => LspSymbolKind::FUNCTION,
    SymbolKind::Field => LspSymbolKind::FIELD,
  }
}

fn map_document_symbol(doc: &Document, symbol: DocumentSymbol) -> LspDocumentSymbol {
  let children = symbol
    .children
    .map(|c| c.into_iter().map(|s| map_document_symbol(doc, s)).collect());

  #[allow(deprecated)]
  LspDocumentSymbol {
    name: symbol.name.clone(),
    detail: None,
    kind: map_symbol_kind(symbol.kind),
    tags: None,
    deprecated: None,
    range: doc.span_to_lsp_range(symbol.range),
    selection_range: doc.span_to_lsp_range(symbol.selection_range),
    children,
  }
}

impl Document {
  pub fn document_symbols(&self) -> Vec<LspDocumentSymbol> {
    let red = self.file.red_tree().clone();
    let root = Root::new(red).unwrap();
    let symbols = collect_symbols(self.file.text(), root);
    symbols
      .into_iter()
      .map(|s| map_document_symbol(self, s))
      .collect()
  }
}

impl Server {
  pub fn document_symbols(&self, uri: &Uri) -> Option<Vec<LspDocumentSymbol>> {
    self.get_document(uri).map(|doc| doc.document_symbols())
  }
}
