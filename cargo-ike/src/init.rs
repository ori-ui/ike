use std::fs;

use cargo_metadata::MetadataCommand;
use clap::Parser;

#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    system: System,
}

#[derive(Parser)]
enum System {
    /// Initialize project files for android.
    Android(Android),
}

#[derive(Parser)]
struct Android {}

impl Command {
    pub fn run(self) -> eyre::Result<()> {
        match self.system {
            System::Android(android) => android.run(),
        }
    }
}

impl Android {
    fn run(self) -> eyre::Result<()> {
        let metadata = MetadataCommand::new().exec()?;

        let root = &metadata.workspace_root;
        let android = root.join("android");
        let main = android.join("src").join("main");
        let ori = main.join("java").join("org").join("ori");

        if android.exists() {
            eprintln!("android project already exists");
            return Ok(());
        }

        fs::create_dir_all(&ori)?;

        fs::write(
            android.join(".gitignore"),
            include_str!("../template/android/.gitignore"),
        )?;

        fs::write(
            android.join("build.gradle"),
            include_str!("../template/android/build.gradle"),
        )?;

        fs::write(
            android.join("settings.gradle"),
            include_str!("../template/android/settings.gradle"),
        )?;

        fs::write(
            main.join("AndroidManifest.xml"),
            include_str!("../template/android/src/main/AndroidManifest.xml"),
        )?;

        fs::write(
            ori.join("RustActivity.java"),
            include_str!("../template/android/src/main/java/org/ori/RustActivity.java"),
        )?;

        fs::write(
            ori.join("RustInputConnection.java"),
            include_str!("../template/android/src/main/java/org/ori/RustInputConnection.java"),
        )?;

        fs::write(
            ori.join("RustView.java"),
            include_str!("../template/android/src/main/java/org/ori/RustView.java"),
        )?;

        eprintln!("android project initialized!");

        Ok(())
    }
}
