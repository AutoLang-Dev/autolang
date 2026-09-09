use crate::FileId;
use async_inc::Query;
use line_index::TextRange;
use syntax::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
  Module,
  Type,
  Function,
  Field,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
  pub name: String,
  pub kind: SymbolKind,
  pub range: TextRange,
  pub selection_range: TextRange,
  pub children: Option<Vec<DocumentSymbol>>,
}

macro_rules! symbol {
  ($src:expr, $node:expr, $kind:expr $(, $children:expr)? $(,)?) => {{
    let node = $node.red().range();
    let name = $node.name().red().range();

    DocumentSymbol {
      name: $src[name].to_string(),
      kind: $kind,
      range: node,
      selection_range: name,
      children: symbol!(@children $($children)?),
    }
  }};
  (@children) => { None };
  (@children $children:expr) => { $children };
}

fn collect_symbols(src: &str, root: Root) -> Vec<DocumentSymbol> {
  root
    .items()
    .into_iter()
    .filter_map(|item| item_symbol(src, item))
    .collect()
}

fn item_symbol(src: &str, item: Item) -> Option<DocumentSymbol> {
  Some(match item {
    Item::Module(m) => module_symbol(src, m),
    Item::Function(f) => function_symbol(src, f),
    Item::Type(t) => type_symbol(src, t),
    _ => return None,
  })
}

fn module_symbol(src: &str, m: ModuleItem) -> DocumentSymbol {
  let children = m
    .items()
    .map(|i| i.into_iter().filter_map(|x| item_symbol(src, x)).collect());

  symbol!(src, m, SymbolKind::Module, children)
}

fn function_symbol(src: &str, f: FunctionItem) -> DocumentSymbol {
  symbol!(src, f, SymbolKind::Function)
}

fn type_symbol(src: &str, t: TypeItem) -> DocumentSymbol {
  let children = if let Type::Record(s) = t.ty() {
    Some(field_symbols(src, s))
  } else {
    None
  };

  symbol!(src, t, SymbolKind::Type, children)
}

fn field_symbols(src: &str, s: RecordType) -> Vec<DocumentSymbol> {
  s.fields()
    .iter()
    .map(|x| symbol!(src, x, SymbolKind::Field))
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GetSymbols(pub FileId);

impl Query for GetSymbols {
  type Value = Vec<DocumentSymbol>;

  async fn compute(self, ctx: async_inc::QueryCtx) -> Self::Value {
    let file = ctx.get(self.0);
    collect_symbols(file.text(), file.root())
  }
}
