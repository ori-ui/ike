mod init;
mod run;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let _ = color_eyre::config::HookBuilder::new()
        .display_env_section(false)
        .display_location_section(false)
        .install();

    let Ike::Ike(args) = Ike::parse();

    if let Err(err) = args.run() {
        eprintln!("{err}");

        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(Parser)]
enum Ike {
    /// Build and run ike projects.
    Ike(Args),
}

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

impl Args {
    fn run(self) -> eyre::Result<()> {
        match self.command {
            Command::Init(init) => init.run(),
            Command::Run(run) => run.run(),
        }
    }
}

#[derive(Parser)]
enum Command {
    /// Initialize a project.
    Init(init::Command),

    /// Run a project.
    #[clap(visible_alias = "r")]
    Run(run::Command),
}
