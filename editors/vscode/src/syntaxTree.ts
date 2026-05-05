import * as path from 'node:path';
import * as vscode from 'vscode';
import { LanguageClient, State } from 'vscode-languageclient/node';
import { getErrorMessage, t } from './config';
import { SYNTAX_TREE_REQUEST, SyntaxTreeParams, SyntaxTreeResult } from './protocol';

export const SYNTAX_TREE_SCHEME = 'autolang-syntax-tree';

export class SyntaxTreeDocumentProvider implements vscode.TextDocumentContentProvider {
  private readonly documents = new Map<string, string>();
  private readonly emitter = new vscode.EventEmitter<vscode.Uri>();
  readonly onDidChange = this.emitter.event;

  provideTextDocumentContent(uri: vscode.Uri): string {
    return this.documents.get(uri.toString()) ?? '';
  }

  update(uri: vscode.Uri, content: string): void {
    this.documents.set(uri.toString(), content);
    this.emitter.fire(uri);
  }

  dispose(): void {
    this.emitter.dispose();
    this.documents.clear();
  }
}

export async function showSyntaxTree(
  client: LanguageClient | undefined,
  provider: SyntaxTreeDocumentProvider,
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== 'autolang') {
    vscode.window.showErrorMessage(
      t('AutoLang syntax tree is only available for AutoLang documents.'),
    );
    return;
  }

  if (!client || client.state !== State.Running) {
    vscode.window.showInformationMessage(t('AutoLang LSP server is not running.'));
    return;
  }

  try {
    const result = await client.sendRequest<SyntaxTreeResult>(SYNTAX_TREE_REQUEST, {
      textDocument: { uri: editor.document.uri.toString() },
    } satisfies SyntaxTreeParams);
    const uri = syntaxTreeUri(editor.document.uri);

    provider.update(uri, result.tree);
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document, {
      preview: false,
      viewColumn: vscode.ViewColumn.Beside,
    });
  } catch (err) {
    vscode.window.showErrorMessage(
      t('Failed to show AutoLang syntax tree: {0}', getErrorMessage(err)),
    );
  }
}

function syntaxTreeUri(sourceUri: vscode.Uri): vscode.Uri {
  const name = `${path.basename(sourceUri.path)}.syntax-tree`;
  return vscode.Uri.from({
    scheme: SYNTAX_TREE_SCHEME,
    path: `/${name}`,
    query: sourceUri.toString(),
  });
}
