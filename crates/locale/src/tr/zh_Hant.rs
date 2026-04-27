pub struct zh_Hant;

impl crate::Language for zh_Hant {
  fn hello_world(&self) -> String {
    "你好，AutoLang！".into()
  }

  fn cli_about(&self) -> String {
    "AutoLang 命令列工具。".into()
  }

  fn cmd_lsp_about(&self) -> String {
    "透過標準輸入輸出啟動 AutoLang 語言伺服器。".into()
  }

  fn arg_help_help(&self) -> String {
    "列印說明資訊。".into()
  }

  fn arg_version_help(&self) -> String {
    "列印版本資訊。".into()
  }

  fn error_heading(&self) -> String {
    "錯誤".into()
  }

  fn error_missing_required_argument(&self) -> String {
    "缺少以下必要參數：".into()
  }

  fn error_unrecognized_subcommand(&self, subcommand: &str) -> String {
    format!("無法識別的子命令 '{subcommand}'")
  }

  fn error_unexpected_argument(&self, argument: &str) -> String {
    format!("發現意外參數 '{argument}'")
  }

  fn error_invalid_command_line(&self) -> String {
    "無效的命令列參數".into()
  }

  fn error_try_help(&self) -> String {
    "更多資訊請嘗試 '--help'。".into()
  }

  fn help_usage_heading(&self) -> String {
    "用法：".into()
  }

  fn help_options_heading(&self) -> String {
    "選項".into()
  }

  fn help_commands_heading(&self) -> String {
    "命令".into()
  }

  fn help_command_value_name(&self) -> String {
    "命令".into()
  }
}
