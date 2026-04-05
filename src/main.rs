use std::process::ExitCode;

use clap::Parser;

use jira_cli::{app, cli::Cli};

fn main() -> ExitCode {
    let cli = Cli::parse();

    match app::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
