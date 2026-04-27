use clap::{Arg, ArgAction, ArgMatches, Command};
use locale::{Language, tr};

const BIN_NAME: &str = "autolang";

const SUBCOMMANDS: &[CommandKind] = &[CommandKind::Lsp];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
  Lsp,
}

impl CommandKind {
  fn name(self) -> &'static str {
    match self {
      Self::Lsp => "lsp",
    }
  }

  fn about(self, tr: &dyn Language) -> String {
    match self {
      Self::Lsp => tr.cmd_lsp_about(),
    }
  }

  fn command(self) -> Command {
    let tr = tr();
    localized_command_base(self.name()).about(self.about(tr))
  }

  fn from_name(name: &str) -> Option<Self> {
    SUBCOMMANDS
      .iter()
      .copied()
      .find(|command| command.name() == name)
  }
}

pub fn root_command() -> Command {
  let tr = tr();

  let mut cmd = localized_command_base(BIN_NAME)
    .version(env!("CARGO_PKG_VERSION"))
    .about(tr.cli_about())
    .disable_version_flag(true)
    .subcommand_required(false)
    .subcommand_value_name(tr.help_command_value_name())
    .subcommand_help_heading(tr.help_commands_heading())
    .arg(version_arg(tr));

  for command in SUBCOMMANDS {
    cmd = cmd.subcommand(command.command());
  }

  cmd
}

pub fn kind_from_matches(matches: &ArgMatches) -> Option<CommandKind> {
  matches.subcommand_name().and_then(CommandKind::from_name)
}

fn localized_command_base(name: &'static str) -> Command {
  let tr = tr();

  Command::new(name)
    .help_template(tr.cli_help_template())
    .disable_help_flag(true)
    .disable_help_subcommand(true)
    .infer_subcommands(false)
    .next_help_heading(tr.help_options_heading())
    .arg(help_arg(tr))
}

fn help_arg(tr: &dyn Language) -> Arg {
  Arg::new("help")
    .short('h')
    .long("help")
    .help(tr.arg_help_help())
    .action(ArgAction::Help)
}

fn version_arg(tr: &dyn Language) -> Arg {
  Arg::new("version")
    .short('V')
    .long("version")
    .help(tr.arg_version_help())
    .action(ArgAction::Version)
}
