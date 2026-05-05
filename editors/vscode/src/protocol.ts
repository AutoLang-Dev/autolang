export const REPARSE_TRACE_NOTIFICATION = 'autolang/reparseTrace';
export const SYNTAX_TREE_REQUEST = 'autolang/syntaxTree';

export type LspPosition = {
  line: number;
  character: number;
};

export type LspRange = {
  start: LspPosition;
  end: LspPosition;
};

export type DirtyRange = {
  old: LspRange;
  new: LspRange;
};

export type ReparseTrace =
  | {
      uri: string;
      strategy: 'noop';
      editRange: LspRange;
    }
  | {
      uri: string;
      strategy: 'token';
      editRange: LspRange;
      dirtyRange: DirtyRange;
    }
  | {
      uri: string;
      strategy: 'node';
      editRange: LspRange;
      dirtyRange: DirtyRange;
      reparser: string;
      oldKind: string;
      newKind: string;
    }
  | {
      uri: string;
      strategy: 'full';
    };

export type SyntaxTreeParams = {
  textDocument: {
    uri: string;
  };
};

export type SyntaxTreeResult = {
  tree: string;
};
