use std::env::set_current_dir;
use std::io::stdout;
use std::process::ExitCode;

use crate::commands::install::{install, uninstall, InstallArgs, UninstallArgs};
use crate::commands::run::{run, RunArgs};
use anyhow::{Error, Result};
use clap::{CommandFactory, Parser, Subcommand};
use tracing::instrument;

mod cfg;
mod checkout;
mod commando;
mod commands;
mod dirs;
mod file_matching;
mod file_set;
mod git;
mod helpers;
mod logging;
mod regex_cache;
mod run_hook;

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(global = true, long, env = "PRECO_TRACING")]
    tracing: bool,

    #[clap(long, hide = true)]
    cwd: Option<String>,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    Run(RunArgs),
    Install(InstallArgs),
    Uninstall(UninstallArgs),
    #[clap(alias = "--generate-shell-completion", hide = true)]
    GenerateShellCompletion {
        shell: clap_complete_command::Shell,
    },
}

#[instrument]
fn run_main() -> Result<ExitCode> {
    let cli = Cli::try_parse()?;

    logging::setup_logging(cli.tracing);
    if let Some(cwd) = &cli.cwd {
        set_current_dir(cwd)?;
    }

    match cli.command {
        Commands::Run(args) => run(&args),
        Commands::Install(args) => install(&args),
        Commands::Uninstall(args) => uninstall(&args),
        Commands::GenerateShellCompletion { shell } => {
            shell.generate(&mut Cli::command(), &mut stdout());
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn main() -> ExitCode {
    run_main().unwrap_or_else(|err| {
        print_error(&err);
        ExitCode::FAILURE
    })
}

fn print_error(err: &Error) {
    let mut causes = err.chain();
    eprintln!("error: {}", causes.next().unwrap());
    for err in causes {
        eprintln!("  Caused by: {}", err);
    }
}
