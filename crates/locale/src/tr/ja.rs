use parser::SyntaxKind;

pub struct ja;

impl crate::Language for ja {
  fn hello_world(&self) -> String {
    "初めまして、AutoLang！".into()
  }

  fn cli_about(&self) -> String {
    "AutoLang コマンドラインツール。".into()
  }

  fn cmd_lsp_about(&self) -> String {
    "標準入出力で AutoLang 言語サーバーを起動します。".into()
  }

  fn arg_help_help(&self) -> String {
    "ヘルプを表示します。".into()
  }

  fn arg_version_help(&self) -> String {
    "バージョンを表示します。".into()
  }

  fn error_heading(&self) -> String {
    "エラー".into()
  }

  fn error_missing_required_argument(&self) -> String {
    "次の必須引数が指定されていません:".into()
  }

  fn error_unrecognized_subcommand(&self, subcommand: &str) -> String {
    format!("認識できないサブコマンド '{subcommand}'")
  }

  fn error_unexpected_argument(&self, argument: &str) -> String {
    format!("予期しない引数 '{argument}' が見つかりました")
  }

  fn error_invalid_command_line(&self) -> String {
    "無効なコマンドライン引数です".into()
  }

  fn error_try_help(&self) -> String {
    "詳しくは '--help' を試してください。".into()
  }

  fn help_usage_heading(&self) -> String {
    "使い方:".into()
  }

  fn help_options_heading(&self) -> String {
    "オプション".into()
  }

  fn help_commands_heading(&self) -> String {
    "コマンド".into()
  }

  fn help_command_value_name(&self) -> String {
    "COMMAND".into()
  }

  fn diagnostic_expected_got(&self, expected: SyntaxKind, actual: SyntaxKind) -> String {
    format!(
      "{} が必要ですが、{} が見つかりました",
      self.syntax_kind_name(expected),
      self.syntax_kind_name(actual)
    )
  }

  fn syntax_kind_name(&self, kind: SyntaxKind) -> String {
    match kind.fixed_text() {
      Some(text) if kind.is_keyword() => format!("キーワード `{text}`"),
      Some(text) => text.into(),
      None => match kind {
        SyntaxKind::Ident => "識別子".into(),
        SyntaxKind::Label => "ラベル".into(),
        SyntaxKind::Int => "整数リテラル".into(),
        SyntaxKind::Char => "文字リテラル".into(),
        SyntaxKind::Byte => "バイトリテラル".into(),
        SyntaxKind::String | SyntaxKind::RawString => "文字列リテラル".into(),
        SyntaxKind::Eof => "ファイルの終端".into(),
        SyntaxKind::Unknown => "不明な token".into(),
        SyntaxKind::UnknownPrefix => "不明なプレフィックス".into(),
        _ => format!("{kind:?}"),
      },
    }
  }
}
