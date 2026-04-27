use insta::{assert_snapshot, with_settings};
use std::process::{Command, Output};

const LOCALE: &str = "zh-Hans";

struct CliCase<'a> {
  name: &'static str,
  args: &'a [&'a str],
}

impl CliCase<'_> {
  fn command_line(&self) -> String {
    let args = if self.args.is_empty() {
      String::new()
    } else {
      format!(" {}", self.args.join(" "))
    };
    format!("AUTOLANG_LOCALE={LOCALE} autolang{args}")
  }

  fn run(&self) -> Output {
    Command::new(env!("CARGO_BIN_EXE_autolang"))
      .args(self.args)
      .env("AUTOLANG_LOCALE", LOCALE)
      .output()
      .expect("failed to run autolang")
  }
}

fn snapshot_cli(case: CliCase<'_>) -> String {
  let output = case.run();
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);
  let status = output
    .status
    .code()
    .map(|code| code.to_string())
    .unwrap_or_else(|| "signal".into());
  let command_line = case.command_line();

  format!("$ {command_line}\n\nstatus: {status}\n\nstdout:\n{stdout}\nstderr:\n{stderr}")
}

macro_rules! snapshot_case {
  ($name:ident, [$($arg:literal),* $(,)?]) => {
    #[test]
    fn $name() {
      let case = CliCase {
        name: stringify!($name),
        args: &[$($arg),*],
      };
      let command_line = case.command_line();

      with_settings!({
        description => command_line.as_str(),
        omit_expression => true,
      }, {
        assert_snapshot!(case.name, snapshot_cli(case));
      });
    }
  };
}

snapshot_case!(no_args_prints_localized_help, []);
snapshot_case!(help_prints_localized_help, ["--help"]);
snapshot_case!(version_prints_package_version, ["-V"]);
snapshot_case!(invalid_subcommand_prints_localized_error, ["nope"]);
snapshot_case!(unknown_root_argument_prints_localized_error_once, ["--bad"]);
snapshot_case!(
  unknown_lsp_argument_uses_clap_subcommand_usage,
  ["lsp", "--bad"]
);
