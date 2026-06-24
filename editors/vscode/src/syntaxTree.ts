import path from 'node:path';
import { LanguageClient, State } from 'vscode-languageclient/node';
import {
  EventEmitter,
  l10n,
  ProviderResult,
  TextDocumentContentProvider,
  Uri,
  ViewColumn,
  window,
  workspace
} from 'vscode';

export const SYNTAX_TREE_REQUEST = 'autolang/syntaxTree';
export const SYNTAX_TREE_SCHEME = 'autolang-syntax-tree';

export type SyntaxTreeParams = {
  textDocument: {
    uri: string;
  };
};

export type SyntaxTreeResult = {
  tree: string;
};

export class SyntaxTreeDocumentProvider implements TextDocumentContentProvider {
  private readonly documents = new Map<string, string>();
  private readonly emitter = new EventEmitter<Uri>();
  readonly onDidChange = this.emitter.event;

  provideTextDocumentContent(uri: Uri): ProviderResult<string> {
    return this.documents.get(uri.toString()) ?? '';
  }

  update(uri: Uri, content: string) {
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
  provider: SyntaxTreeDocumentProvider
) {
  const editor = window.activeTextEditor;
  if (!editor || editor.document.languageId != 'autolang') {
    window.showErrorMessage(
      l10n.t('AutoLang syntax tree is only available for AutoLang documents.'),
    );
    return;
  }

  if (!client || client.state != State.Running) {
    window.showInformationMessage(
      l10n.t('AutoLang LSP server is not running.'),
    );
    return;
  }

  try {
    const result = await client.sendRequest<SyntaxTreeResult>(SYNTAX_TREE_REQUEST, {
      textDocument: { uri: editor.document.uri.toString() },
    } satisfies SyntaxTreeParams);
    const uri = syntaxTreeUri(editor.document.uri);

    provider.update(uri, result.tree);
    const document = await workspace.openTextDocument(uri);
    await window.showTextDocument(document, {
      preview: false,
      viewColumn: ViewColumn.Beside,
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    window.showErrorMessage(
      l10n.t('Failed to show AutoLang syntax tree: {0}', message),
    );
  }
}

function syntaxTreeUri(sourceUri: Uri): Uri {
  const name = `${path.basename(sourceUri.path)}.syntax-tree`;
  return Uri.from({
    scheme: SYNTAX_TREE_SCHEME,
    path: `/${name}`,
    query: sourceUri.toString(),
  });
}
