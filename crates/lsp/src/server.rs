mod diagnostic;
mod document_symbols;
mod reparse_trace;
mod semantic_tokens;
mod syntax_tree;

pub use {reparse_trace::*, syntax_tree::*};

use crate::server::{
  diagnostic::document_diagnostics,
  document_symbols::document_symbols,
  semantic_tokens::{semantic_tokens, tokens_legend},
};
use async_inc::Database;
use async_lsp::{ClientSocket, LanguageServer, ResponseError, lsp_types::*, router::Router};
use ide::{FileId, SourceFile};
use line_index::{LineIndex, TextRange, TextSize, WideEncoding, WideLineCol};
use std::{collections::HashMap, ops::ControlFlow, pin::Pin};
use syntax::Indel;

fn lsp_range_to_span(index: &LineIndex, range: Range) -> TextRange {
  let start = lsp_position_to_offset(index, range.start);
  let end = lsp_position_to_offset(index, range.end);
  TextRange::new(start, end)
}

fn span_to_lsp_range(index: &LineIndex, range: TextRange) -> Range {
  let start = offset_to_lsp_position(index, range.start());
  let end = offset_to_lsp_position(index, range.end());
  Range::new(start, end)
}

fn lsp_position_to_offset(index: &LineIndex, pos: Position) -> TextSize {
  let line_col = WideLineCol {
    line: pos.line,
    col: pos.character,
  };
  let line_col = index.to_utf8(WideEncoding::Utf16, line_col).unwrap();
  index.offset(line_col).unwrap()
}

fn offset_to_lsp_position(index: &LineIndex, offset: TextSize) -> Position {
  let line_col = index.line_col(offset);
  let line_col = index.to_wide(WideEncoding::Utf16, line_col).unwrap();
  Position::new(line_col.line, line_col.col)
}

pub struct Server {
  client: ClientSocket,
  vfs: HashMap<Url, FileId>,
  db: Database,
}

type ResponseFuture<T> = Pin<Box<dyn Future<Output = Result<T, ResponseError>> + Send + 'static>>;

impl LanguageServer for Server {
  type Error = ResponseError;
  type NotifyResult = ControlFlow<async_lsp::Result<()>>;

  fn initialize(&mut self, _: InitializeParams) -> ResponseFuture<InitializeResult> {
    Box::pin(async {
      Ok(InitializeResult {
        capabilities: Self::capabilities(),
        server_info: None,
      })
    })
  }

  fn did_open(&mut self, params: DidOpenTextDocumentParams) -> Self::NotifyResult {
    let (id, _) = self.db.create(SourceFile::new(params.text_document.text));
    self.vfs.insert(params.text_document.uri, id);
    ControlFlow::Continue(())
  }

  fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Self::NotifyResult {
    let uri = params.text_document.uri;
    let id = *self.vfs.get(&uri).unwrap();
    let mut file = (*self.db.snapshot().get(id)).clone();

    let mut traces = vec![];

    for change in params.content_changes {
      let insert = change.text;
      let Some(range) = change.range else {
        file.set_text(insert);
        continue;
      };

      let index = file.index();
      let delete = lsp_range_to_span(index, range);
      let indel = Indel { delete, insert };

      let Some((new, old)) = file.apply_change(&indel) else {
        continue;
      };
      file = new;

      let trace = old.range();
      let trace = TextRange::at(
        trace.start(),
        trace.len() - indel.delete.len() + indel.insert_len(),
      );
      traces.push(span_to_lsp_range(file.index(), trace));
    }

    self.db.set(id, file).unwrap();

    {
      let params = ReparseTraceParams::new(uri, traces);
      let _ = self.client.notify::<ReparseTraceNotification>(params);
    }

    ControlFlow::Continue(())
  }

  fn did_close(&mut self, params: DidCloseTextDocumentParams) -> Self::NotifyResult {
    self.vfs.remove(&params.text_document.uri);
    ControlFlow::Continue(())
  }

  fn did_save(&mut self, _: DidSaveTextDocumentParams) -> Self::NotifyResult {
    ControlFlow::Continue(())
  }

  fn semantic_tokens_full(
    &mut self,
    params: SemanticTokensParams,
  ) -> ResponseFuture<Option<SemanticTokensResult>> {
    let id = *self.vfs.get(&params.text_document.uri).unwrap();
    let ctx = self.db.query_ctx();
    Box::pin(async move {
      Ok(Some(SemanticTokensResult::Tokens(
        semantic_tokens(ctx, id).await,
      )))
    })
  }

  fn document_symbol(
    &mut self,
    params: DocumentSymbolParams,
  ) -> ResponseFuture<Option<DocumentSymbolResponse>> {
    let id = *self.vfs.get(&params.text_document.uri).unwrap();
    let ctx = self.db.query_ctx();
    Box::pin(async move {
      Ok(Some(DocumentSymbolResponse::Nested(
        document_symbols(ctx, id).await,
      )))
    })
  }

  fn document_diagnostic(
    &mut self,
    params: DocumentDiagnosticParams,
  ) -> ResponseFuture<DocumentDiagnosticReportResult> {
    let id = *self.vfs.get(&params.text_document.uri).unwrap();
    let ctx = self.db.query_ctx();
    Box::pin(async move {
      Ok(DocumentDiagnosticReportResult::Report(
        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
          related_documents: None,
          full_document_diagnostic_report: FullDocumentDiagnosticReport {
            result_id: None,
            items: document_diagnostics(ctx, id).await,
          },
        }),
      ))
    })
  }
}

impl Server {
  pub fn router(client: ClientSocket) -> Router<Self> {
    let mut router = Router::from_language_server(Self {
      client,
      vfs: HashMap::new(),
      db: Database::new(),
    });

    router.request::<SyntaxTreeRequest, _>(|server, params| {
      let id = *server.vfs.get(&params.text_document.uri).unwrap();
      let ctx = server.db.query_ctx();
      async move { Ok(syntax_tree(ctx, id).await) }
    });

    router
  }

  pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
      text_document_sync: Some(TextDocumentSyncCapability::Kind(
        TextDocumentSyncKind::INCREMENTAL,
      )),
      semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
        SemanticTokensOptions {
          legend: tokens_legend(),
          full: Some(SemanticTokensFullOptions::Delta { delta: Some(false) }),
          ..Default::default()
        },
      )),
      document_symbol_provider: Some(OneOf::Left(true)),
      diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
        identifier: Some("autolang".into()),
        ..Default::default()
      })),
      ..Default::default()
    }
  }
}
