use crate::server::span_to_lsp_range;
use async_inc::{Query, QueryCtx};
use async_lsp::lsp_types::{DocumentSymbol as LspDocumentSymbol, SymbolKind as LspSymbolKind};
use ide::{DocumentSymbol, FileId, GetSymbols, SymbolKind};
use line_index::LineIndex;

fn map_symbol_kind(kind: SymbolKind) -> LspSymbolKind {
  match kind {
    SymbolKind::Module => LspSymbolKind::MODULE,
    SymbolKind::Type => LspSymbolKind::STRUCT,
    SymbolKind::Function => LspSymbolKind::FUNCTION,
  }
}

fn map_document_symbol(index: &LineIndex, symbol: &DocumentSymbol) -> LspDocumentSymbol {
  let children = symbol
    .children
    .as_ref()
    .map(|c| c.iter().map(|s| map_document_symbol(index, s)).collect());

  #[allow(deprecated)]
  LspDocumentSymbol {
    name: symbol.name.clone(),
    detail: None,
    kind: map_symbol_kind(symbol.kind),
    tags: None,
    deprecated: None,
    range: span_to_lsp_range(index, symbol.range),
    selection_range: span_to_lsp_range(index, symbol.selection_range),
    children,
  }
}

pub async fn document_symbols(ctx: QueryCtx, id: FileId) -> Vec<LspDocumentSymbol> {
  let file = ctx.get(id);
  let index = file.index();
  GetSymbols(id)
    .execute(ctx)
    .await
    .iter()
    .map(|s| map_document_symbol(index, s))
    .collect()
}
