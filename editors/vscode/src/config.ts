import * as vscode from 'vscode';

export const CLI_PATH_SETTING = 'cli.path';
export const DEV_RUN_SERVER_FROM_SOURCE_SETTING = 'dev.runServerFromSource';
export const DEBUG_REPARSE_TRACE_SETTING = 'debug.reparseTrace';

export function t(message: string, ...args: Array<string | number | boolean>): string {
  return vscode.l10n.t(message, ...args);
}

export function getErrorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

export function isReparseTraceEnabled(): boolean {
  return vscode.workspace
    .getConfiguration('autolang')
    .get<boolean>(DEBUG_REPARSE_TRACE_SETTING, false);
}
