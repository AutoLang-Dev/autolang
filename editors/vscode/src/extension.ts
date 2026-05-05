import * as vscode from 'vscode';
import { DEBUG_REPARSE_TRACE_SETTING, isReparseTraceEnabled, t } from './config';
import { LanguageClientController } from './client';
import { ReparseFlash } from './reparseTrace';
import { showSyntaxTree, SYNTAX_TREE_SCHEME, SyntaxTreeDocumentProvider } from './syntaxTree';

let clientController: LanguageClientController | undefined;

export function activate(context: vscode.ExtensionContext) {
  const reparseFlash = new ReparseFlash();
  const syntaxTreeProvider = new SyntaxTreeDocumentProvider();
  clientController = new LanguageClientController(reparseFlash);

  context.subscriptions.push(
    reparseFlash,
    syntaxTreeProvider,
    vscode.workspace.registerTextDocumentContentProvider(
      SYNTAX_TREE_SCHEME,
      syntaxTreeProvider,
    ),
    vscode.commands.registerCommand('autolang.startServer', () =>
      clientController?.start(context),
    ),
    vscode.commands.registerCommand('autolang.stopServer', () => clientController?.stop()),
    vscode.commands.registerCommand('autolang.restartServer', () =>
      clientController?.restart(context),
    ),
    vscode.commands.registerCommand('autolang.toggleReparseTrace', () => toggleReparseTrace()),
    vscode.commands.registerCommand('autolang.showSyntaxTree', () =>
      showSyntaxTree(clientController?.currentClient, syntaxTreeProvider),
    ),
  );

  clientController.start(context);
}

export async function deactivate(): Promise<void> {
  await clientController?.deactivate();
  clientController = undefined;
}

async function toggleReparseTrace(): Promise<void> {
  const enabled = isReparseTraceEnabled();
  await vscode.workspace
    .getConfiguration('autolang')
    .update(DEBUG_REPARSE_TRACE_SETTING, !enabled, vscode.ConfigurationTarget.Global);
  vscode.window.showInformationMessage(
    !enabled ? t('AutoLang reparse trace enabled.') : t('AutoLang reparse trace disabled.'),
  );
}
