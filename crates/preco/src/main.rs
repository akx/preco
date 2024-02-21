use std::io::stdout;
use std::process::ExitCode;

use crate::commands::run::run;
use anyhow::{Error, Result};
use clap::{CommandFactory, Parser, Subcommand};
use commands::run::RunArgs;
use tracing::instrument;

mod cfg;
mod checkout;
mod commands;
mod file_matching;
mod file_set;
mod git;
mod helpers;
mod logging;
mod run_hook;

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(global = true, long, env = "PRECO_TRACING")]
    tracing: bool,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    Run(RunArgs),
    #[clap(alias = "--generate-shell-completion", hide = true)]
    GenerateShellCompletion {
        shell: clap_complete_command::Shell,
    },
}

#[instrument]
async fn run_main() -> Result<ExitCode> {
    let cli = Cli::try_parse()?;

    logging::setup_logging(cli.tracing);

    match cli.command {
        Commands::Run(args) => run(&args),
        Commands::GenerateShellCompletion { shell } => {
            shell.generate(&mut Cli::command(), &mut stdout());
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn main() -> ExitCode {
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the runtime")
        .block_on(run_main());

    result.unwrap_or_else(|err| {
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
