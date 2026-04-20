use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about = "Workspace automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Placeholder quality command for future custom checks.
    Quality,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Quality => {
            println!("xtask quality: no extra checks yet");
        }
    }
}
