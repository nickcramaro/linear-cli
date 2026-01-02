mod client;
mod commands;
mod error;
mod generated;
mod output;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle NO_COLOR env var and --no-color flag
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        owo_colors::set_override(false);
    }

    if let Err(e) = run(cli).await {
        output::print_error(&e);
        std::process::exit(e.exit_code());
    }
}

async fn run(cli: Cli) -> error::Result<()> {
    let client = client::LinearClient::from_env()?;

    match cli.command {
        Commands::User { command } => match command {
            commands::user::UserCommands::Me => {
                commands::user::handle_me(&client).await?;
            }
        },
    }

    Ok(())
}
