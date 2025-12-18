mod init;

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
        }
    }
}

#[derive(Parser)]
enum Command {
    /// Initialize a project.
    Init(init::Command),
}
