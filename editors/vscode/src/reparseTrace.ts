import * as vscode from 'vscode';
import { isReparseTraceEnabled } from './config';
import { LspRange, ReparseTrace } from './protocol';

export class ReparseFlash implements vscode.Disposable {
  private readonly tokenDecoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: new vscode.ThemeColor('editor.wordHighlightBackground'),
    border: '1px solid rgba(80, 200, 120, 0.8)',
  });
  private readonly nodeDecoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: new vscode.ThemeColor('editor.selectionHighlightBackground'),
    border: '1px solid rgba(80, 160, 255, 0.85)',
  });
  private readonly fullDecoration = vscode.window.createTextEditorDecorationType({
    border: '1px solid rgba(255, 170, 60, 0.95)',
  });
  private timer: NodeJS.Timeout | undefined;

  show(trace: ReparseTrace): void {
    if (!isReparseTraceEnabled()) {
      return;
    }

    const editor = vscode.window.visibleTextEditors.find(
      (visibleEditor) => visibleEditor.document.uri.toString() === trace.uri,
    );
    if (!editor) {
      return;
    }

    const range = visibleTraceRange(editor.document, trace);
    const decoration = this.decoration(trace.strategy);

    this.clear();
    editor.setDecorations(decoration, [range]);
    this.timer = setTimeout(() => this.clear(), flashDuration(trace.strategy));
  }

  dispose(): void {
    this.clear();
    this.tokenDecoration.dispose();
    this.nodeDecoration.dispose();
    this.fullDecoration.dispose();
  }

  private clear(): void {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = undefined;
    }

    for (const editor of vscode.window.visibleTextEditors) {
      editor.setDecorations(this.tokenDecoration, []);
      editor.setDecorations(this.nodeDecoration, []);
      editor.setDecorations(this.fullDecoration, []);
    }
  }

  private decoration(strategy: ReparseTrace['strategy']): vscode.TextEditorDecorationType {
    switch (strategy) {
      case 'noop':
      case 'token':
        return this.tokenDecoration;
      case 'node':
        return this.nodeDecoration;
      case 'full':
        return this.fullDecoration;
    }
  }
}

function visibleTraceRange(document: vscode.TextDocument, trace: ReparseTrace): vscode.Range {
  switch (trace.strategy) {
    case 'token':
    case 'node':
      return toVisibleRange(document, trace.dirtyRange.new);
    case 'noop':
      return toVisibleRange(document, trace.editRange);
    case 'full':
      return fullDocumentVisibleRange(document);
  }
}

function flashDuration(strategy: ReparseTrace['strategy']): number {
  switch (strategy) {
    case 'noop':
    case 'token':
      return 350;
    case 'node':
      return 500;
    case 'full':
      return 700;
  }
}

function toVisibleRange(document: vscode.TextDocument, lspRange: LspRange): vscode.Range {
  const start = new vscode.Position(lspRange.start.line, lspRange.start.character);
  let end = new vscode.Position(lspRange.end.line, lspRange.end.character);

  if (end.isBeforeOrEqual(start)) {
    const offset = document.offsetAt(start);
    if (offset < document.getText().length) {
      end = document.positionAt(offset + 1);
    } else {
      return new vscode.Range(document.positionAt(Math.max(0, offset - 1)), start);
    }
  }

  return new vscode.Range(start, end);
}

function fullDocumentVisibleRange(document: vscode.TextDocument): vscode.Range {
  const start = new vscode.Position(0, 0);
  const end = document.positionAt(document.getText().length);
  return new vscode.Range(start, end);
}
