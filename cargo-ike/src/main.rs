mod init;
mod run;

use clap::Parser;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let Ike::Ike(args) = Ike::parse();

    args.run()
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
