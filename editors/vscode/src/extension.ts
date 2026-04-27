import * as vscode from 'vscode';
import { ExtensionMode } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  State,
} from 'vscode-languageclient/node';
import * as path from 'node:path';

let client: LanguageClient | undefined;

const CLI_PATH_SETTING = 'cli.path';
const DEV_RUN_SERVER_FROM_SOURCE_SETTING = 'dev.runServerFromSource';

type ServerCommand = {
  command: string;
  args: string[];
  options?: {
    cwd?: string;
  };
};

function t(message: string, ...args: Array<string | number | boolean>): string {
  return vscode.l10n.t(message, ...args);
}

function getErrorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function getSourceServerCommand(context: vscode.ExtensionContext): ServerCommand {
  // extensionUri points to editors/vscode, go up two levels to get the repo root
  const repoRoot = path.resolve(context.extensionUri.fsPath, '..', '..');
  return { command: 'cargo', args: ['run', '--', 'lsp'], options: { cwd: repoRoot } };
}

function getCliCommand(): ServerCommand {
  const config = vscode.workspace.getConfiguration('autolang');
  const cliPath = config.get<string>(CLI_PATH_SETTING)?.trim();

  if (cliPath) {
    return { command: cliPath, args: ['lsp'] };
  }

  throw new Error(
    t('AutoLang CLI path is not configured. Please set "autolang.cli.path" in settings.'),
  );
}

function getServerCommand(context: vscode.ExtensionContext): ServerCommand {
  const config = vscode.workspace.getConfiguration('autolang');
  const runServerFromSource = config.get<boolean>(DEV_RUN_SERVER_FROM_SOURCE_SETTING, true);

  if (context.extensionMode === ExtensionMode.Development && runServerFromSource) {
    return getSourceServerCommand(context);
  }

  return getCliCommand();
}

async function startServer(
  context: vscode.ExtensionContext,
): Promise<void> {
  if (client?.state === State.Running || client?.state === State.Starting) {
    vscode.window.showInformationMessage(
      t('AutoLang LSP server is already running.'),
    );
    return;
  }

  if (client) {
    client.dispose();
    client = undefined;
  }

  try {
    const serverCommand = getServerCommand(context);

    const serverOptions: ServerOptions = {
      run: serverCommand,
      debug: serverCommand,
    };

    const clientOptions: LanguageClientOptions = {
      documentSelector: [{ language: 'autolang' }],
    };

    client = new LanguageClient(
      'autolang',
      'AutoLang Language Server',
      serverOptions,
      clientOptions,
    );

    await client.start();
    vscode.window.showInformationMessage(t('AutoLang LSP server started.'));
  } catch (err) {
    client = undefined;
    vscode.window.showErrorMessage(
      t('Failed to start AutoLang LSP server: {0}', getErrorMessage(err)),
    );
  }
}

async function stopServer(): Promise<boolean> {
  if (!client || client.state !== State.Running) {
    vscode.window.showInformationMessage(
      t('AutoLang LSP server is not running.'),
    );
    return true;
  }

  try {
    await client.stop();
    client = undefined;
    vscode.window.showInformationMessage(t('AutoLang LSP server stopped.'));
    return true;
  } catch (err) {
    vscode.window.showErrorMessage(
      t('Failed to stop AutoLang LSP server: {0}', getErrorMessage(err)),
    );
    return false;
  }
}

async function restartServer(
  context: vscode.ExtensionContext,
): Promise<void> {
  if (await stopServer()) {
    await startServer(context);
  }
}

export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand('autolang.startServer', () =>
      startServer(context),
    ),
    vscode.commands.registerCommand('autolang.stopServer', () =>
      stopServer(),
    ),
    vscode.commands.registerCommand('autolang.restartServer', () =>
      restartServer(context),
    ),
  );

  startServer(context);
}

export async function deactivate(): Promise<void> {
  if (!client) {
    return;
  }

  try {
    await client.stop();
  } finally {
    client = undefined;
  }
}
