use clap::{
  Error,
  error::{ContextKind, ContextValue, ErrorKind},
};
use locale::tr;

pub fn exit_with_clap_error(err: Error) -> ! {
  if should_exit_with_clap_display(&err) {
    err.exit();
  }

  eprintln!("{}: {}", tr().error_heading(), error_message(&err));
  if let Some(ContextValue::StyledStr(usage)) = err.get(ContextKind::Usage) {
    eprintln!();
    eprintln!("{}", localized_usage(usage));
  }
  eprintln!();
  eprintln!("{}", tr().error_try_help());
  std::process::exit(err.exit_code());
}

fn should_exit_with_clap_display(err: &Error) -> bool {
  matches!(
    err.kind(),
    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
  )
}

fn error_message(err: &Error) -> String {
  let tr = tr();
  match err.kind() {
    ErrorKind::MissingRequiredArgument => {
      let mut message = tr.error_missing_required_argument();
      if let Some(args) = context_strings(err, ContextKind::InvalidArg) {
        for arg in args {
          message.push_str("\n  ");
          message.push_str(arg);
        }
      }
      message
    }
    ErrorKind::UnknownArgument => context_string(err, ContextKind::InvalidArg)
      .map(|argument| tr.error_unexpected_argument(argument))
      .unwrap_or_else(fallback_error_message),
    ErrorKind::InvalidSubcommand => context_string(err, ContextKind::InvalidSubcommand)
      .map(|subcommand| tr.error_unrecognized_subcommand(subcommand))
      .unwrap_or_else(fallback_error_message),
    _ => fallback_error_message(),
  }
}

fn context_string(err: &Error, kind: ContextKind) -> Option<&str> {
  match err.get(kind) {
    Some(ContextValue::String(value)) => Some(value),
    _ => None,
  }
}

fn context_strings(err: &Error, kind: ContextKind) -> Option<&[String]> {
  match err.get(kind) {
    Some(ContextValue::Strings(values)) => Some(values),
    _ => None,
  }
}

fn fallback_error_message() -> String {
  tr().error_invalid_command_line()
}

fn localized_usage(usage: &clap::builder::StyledStr) -> String {
  let usage = usage.to_string();

  // clap formats usage inside errors with its built-in English heading even
  // when the help template is localized, so replace just that fixed prefix.
  if let Some(rest) = usage.strip_prefix("Usage:") {
    format!("{}{}", tr().help_usage_heading(), rest)
  } else {
    usage
  }
}
