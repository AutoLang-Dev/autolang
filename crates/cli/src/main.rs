mod cli;

use cli::CommandKind;

fn main() -> anyhow::Result<()> {
  match cli::parse()? {
    Some(CommandKind::Lsp) => lsp::run(),
    None => Ok(()),
  }
}
