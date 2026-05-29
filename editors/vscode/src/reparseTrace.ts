import { Disposable, TextDocument, ThemeColor, window, Range as VscRange, Position } from "vscode";
import { Range, TextDocumentIdentifier } from "vscode-languageclient";

export const REPARSE_TRACE_NOTIFICATION = 'autolang/reparseTrace';

export type ReparseTraceParams = {
  textDocument: TextDocumentIdentifier,
  ranges: Range[],
}

export class ReparseFlash implements Disposable {
  private readonly decoration = window.createTextEditorDecorationType({
    backgroundColor: new ThemeColor('editor.selectionHighlightBackground'),
    border: '1px solid rgba(80, 160, 255, 0.85)',
  });

  private timer: NodeJS.Timeout | undefined;

  show(trace: ReparseTraceParams) {
    const editor = window.visibleTextEditors.find(
      ve => ve.document.uri.toString() == trace.textDocument.uri,
    );
    if (!editor) {
      return;
    }

    const ranges = trace.ranges.map(
      range => toVisibleRange(editor.document, range),
    );

    this.clear();
    editor.setDecorations(this.decoration, ranges);
    this.timer = setTimeout(() => this.clear(), 500);
  }

  private clear() {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = undefined;
    }

    for (const editor of window.visibleTextEditors) {
      editor.setDecorations(this.decoration, []);
    }
  }

  dispose() {
    this.clear();
    this.decoration.dispose();
  }
}

function toVisibleRange(document: TextDocument, range: Range): VscRange {
  const start = new Position(range.start.line, range.start.character);
  let end = new Position(range.end.line, range.end.character);

  if (end.isBeforeOrEqual(start)) {
    const offset = document.offsetAt(start);
    if (offset < document.getText().length) {
      end = document.positionAt(offset + 1);
    } else {
      return new VscRange(document.positionAt(Math.max(0, offset - 1)), start);
    }
  }

  return new VscRange(start, end);
}
