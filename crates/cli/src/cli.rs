mod command;
mod error;

pub use command::CommandKind;

pub fn parse() -> anyhow::Result<Option<CommandKind>> {
  if std::env::args_os().len() == 1 {
    let mut cmd = command::root_command();
    cmd.print_help()?;
    println!();
    return Ok(None);
  }

  let matches = command::root_command()
    .try_get_matches()
    .unwrap_or_else(|err| error::exit_with_clap_error(err));
  Ok(command::kind_from_matches(&matches))
}
