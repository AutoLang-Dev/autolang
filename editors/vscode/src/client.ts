import * as path from 'node:path';
import * as vscode from 'vscode';
import { ExtensionMode } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  State,
} from 'vscode-languageclient/node';
import {
  CLI_PATH_SETTING,
  DEV_RUN_SERVER_FROM_SOURCE_SETTING,
  getErrorMessage,
  t,
} from './config';
import { REPARSE_TRACE_NOTIFICATION, ReparseTrace } from './protocol';
import { ReparseFlash } from './reparseTrace';

type ServerCommand = {
  command: string;
  args: string[];
  options?: {
    cwd?: string;
  };
};

export class LanguageClientController {
  private client: LanguageClient | undefined;

  constructor(private readonly reparseFlash: ReparseFlash) {}

  get currentClient(): LanguageClient | undefined {
    return this.client;
  }

  async start(context: vscode.ExtensionContext): Promise<void> {
    if (this.client?.state === State.Running || this.client?.state === State.Starting) {
      vscode.window.showInformationMessage(t('AutoLang LSP server is already running.'));
      return;
    }

    if (this.client) {
      this.client.dispose();
      this.client = undefined;
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

      this.client = new LanguageClient(
        'autolang',
        'AutoLang Language Server',
        serverOptions,
        clientOptions,
      );

      await this.client.start();
      this.client.onNotification(REPARSE_TRACE_NOTIFICATION, (trace: ReparseTrace) => {
        this.reparseFlash.show(trace);
      });
      vscode.window.showInformationMessage(t('AutoLang LSP server started.'));
    } catch (err) {
      this.client = undefined;
      vscode.window.showErrorMessage(
        t('Failed to start AutoLang LSP server: {0}', getErrorMessage(err)),
      );
    }
  }

  async stop(): Promise<boolean> {
    if (!this.client || this.client.state !== State.Running) {
      vscode.window.showInformationMessage(t('AutoLang LSP server is not running.'));
      return true;
    }

    try {
      await this.client.stop();
      this.client = undefined;
      vscode.window.showInformationMessage(t('AutoLang LSP server stopped.'));
      return true;
    } catch (err) {
      vscode.window.showErrorMessage(
        t('Failed to stop AutoLang LSP server: {0}', getErrorMessage(err)),
      );
      return false;
    }
  }

  async restart(context: vscode.ExtensionContext): Promise<void> {
    if (await this.stop()) {
      await this.start(context);
    }
  }

  async deactivate(): Promise<void> {
    if (!this.client) {
      return;
    }

    try {
      await this.client.stop();
    } finally {
      this.client = undefined;
    }
  }
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
