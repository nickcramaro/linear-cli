mod client;
mod commands;
mod error;
mod generated;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::User { command } => match command {
            commands::user::UserCommands::Me => {
                println!("user me - not implemented yet");
            }
        },
    }
}
