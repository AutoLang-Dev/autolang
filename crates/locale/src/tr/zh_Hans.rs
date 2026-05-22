use parser::SyntaxKind;

pub struct zh_Hans;

impl crate::Language for zh_Hans {
  fn hello_world(&self) -> String {
    "你好，AutoLang！".into()
  }

  fn cli_about(&self) -> String {
    "AutoLang 命令行工具。".into()
  }

  fn cmd_lsp_about(&self) -> String {
    "通过标准输入输出启动 AutoLang 语言服务器。".into()
  }

  fn arg_help_help(&self) -> String {
    "打印帮助信息。".into()
  }

  fn arg_version_help(&self) -> String {
    "打印版本信息。".into()
  }

  fn error_heading(&self) -> String {
    "错误".into()
  }

  fn error_missing_required_argument(&self) -> String {
    "缺少以下必需参数：".into()
  }

  fn error_unrecognized_subcommand(&self, subcommand: &str) -> String {
    format!("无法识别的子命令 '{subcommand}'")
  }

  fn error_unexpected_argument(&self, argument: &str) -> String {
    format!("发现意外参数 '{argument}'")
  }

  fn error_invalid_command_line(&self) -> String {
    "无效的命令行参数".into()
  }

  fn error_try_help(&self) -> String {
    "更多信息请尝试 '--help'。".into()
  }

  fn help_usage_heading(&self) -> String {
    "用法：".into()
  }

  fn help_options_heading(&self) -> String {
    "选项".into()
  }

  fn help_commands_heading(&self) -> String {
    "命令".into()
  }

  fn help_command_value_name(&self) -> String {
    "命令".into()
  }

  fn diagnostic_expected_got(&self, expected: SyntaxKind, actual: SyntaxKind) -> String {
    format!(
      "期望 {}，但遇到 {}",
      self.syntax_kind_name(expected),
      self.syntax_kind_name(actual)
    )
  }

  fn syntax_kind_name(&self, kind: SyntaxKind) -> String {
    match kind.fixed_text() {
      Some(text) if kind.is_keyword() => format!("关键字 `{text}`"),
      Some(text) => text.into(),
      None => match kind {
        SyntaxKind::Ident => "标识符".into(),
        SyntaxKind::Label => "标签".into(),
        SyntaxKind::Int => "整数字面量".into(),
        SyntaxKind::Char => "字符字面量".into(),
        SyntaxKind::Byte => "字节字面量".into(),
        SyntaxKind::String | SyntaxKind::RawString => "字符串字面量".into(),
        SyntaxKind::Eof => "文件结尾".into(),
        SyntaxKind::Unknown => "未知 token".into(),
        SyntaxKind::UnknownPrefix => "未知前缀".into(),
        _ => format!("{kind:?}"),
      },
    }
  }
}
