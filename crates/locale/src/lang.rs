use std::sync::RwLock;
use unic_langid::{LanguageIdentifier, lang, langid, script};

use super::tr::*;

pub trait Language: Sync {
  fn hello_world(&self) -> String {
    "Hello AutoLang!".into()
  }

  fn cli_about(&self) -> String {
    "AutoLang command line tools.".into()
  }

  fn cmd_lsp_about(&self) -> String {
    "Start the AutoLang language server over stdio.".into()
  }

  fn arg_help_help(&self) -> String {
    "Print help.".into()
  }

  fn arg_version_help(&self) -> String {
    "Print version.".into()
  }

  fn error_heading(&self) -> String {
    "error".into()
  }

  fn error_missing_required_argument(&self) -> String {
    "the following required arguments were not provided:".into()
  }

  fn error_unrecognized_subcommand(&self, subcommand: &str) -> String {
    format!("unrecognized subcommand '{subcommand}'")
  }

  fn error_unexpected_argument(&self, argument: &str) -> String {
    format!("unexpected argument '{argument}' found")
  }

  fn error_invalid_command_line(&self) -> String {
    "invalid command line".into()
  }

  fn error_try_help(&self) -> String {
    "For more information, try '--help'.".into()
  }

  fn help_usage_heading(&self) -> String {
    "Usage:".into()
  }

  fn help_options_heading(&self) -> String {
    "Options".into()
  }

  fn help_commands_heading(&self) -> String {
    "Commands".into()
  }

  fn help_command_value_name(&self) -> String {
    "COMMAND".into()
  }

  fn cli_help_template(&self) -> String {
    format!(
      "{{before-help}}{{about-with-newline}}\n{} {{usage}}\n\n{{all-args}}{{after-help}}",
      self.help_usage_heading()
    )
  }
}

pub fn tr_of(locale: String) -> &'static dyn Language {
  let LanguageIdentifier {
    language: lang,
    script,
    ..
  } = locale.parse().unwrap_or(langid!("en-US"));

  if lang == lang!("en") {
    &en_US::en_US
  } else if lang == lang!("ja") {
    &ja::ja
  } else if lang == lang!("zh") {
    if script == Some(script!("Hant")) {
      &zh_Hant::zh_Hant
    } else {
      &zh_Hans::zh_Hans
    }
  } else {
    &en_US::en_US
  }
}

static TR: RwLock<Option<&'static dyn Language>> = RwLock::new(None);

pub fn set_tr(locale: String) {
  *TR.write().unwrap() = Some(tr_of(locale));
}

pub fn tr() -> &'static dyn Language {
  if let Some(lang) = *TR.read().unwrap() {
    return lang;
  }

  let locale = std::env::var("AUTOLANG_LOCALE")
    .ok()
    .filter(|locale| !locale.trim().is_empty())
    .or_else(crate::get_locale)
    .unwrap_or_else(|| "en-US".into());

  set_tr(locale);
  TR.read().unwrap().unwrap()
}
