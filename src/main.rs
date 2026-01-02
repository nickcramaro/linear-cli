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
        Commands::Issue { command } => match command {
            commands::issue::IssueCommands::List(args) => {
                commands::issue::handle_list(&client, &args).await?;
            }
            commands::issue::IssueCommands::Get(args) => {
                commands::issue::handle_get(&client, &args).await?;
            }
            commands::issue::IssueCommands::Create(args) => {
                commands::issue::handle_create(&client, &args).await?;
            }
            commands::issue::IssueCommands::Update(args) => {
                commands::issue::handle_update(&client, &args).await?;
            }
        },
        Commands::Team { command } => match command {
            commands::team::TeamCommands::List => {
                commands::team::handle_list(&client).await?;
            }
            commands::team::TeamCommands::Get(args) => {
                commands::team::handle_get(&client, &args).await?;
            }
        },
        Commands::Project { command } => match command {
            commands::project::ProjectCommands::List(args) => {
                commands::project::handle_list(&client, &args).await?;
            }
            commands::project::ProjectCommands::Get(args) => {
                commands::project::handle_get(&client, &args).await?;
            }
            commands::project::ProjectCommands::Create(args) => {
                commands::project::handle_create(&client, &args).await?;
            }
        },
        Commands::Cycle { command } => match command {
            commands::cycle::CycleCommands::List(args) => {
                commands::cycle::handle_list(&client, &args).await?;
            }
            commands::cycle::CycleCommands::Get(args) => {
                commands::cycle::handle_get(&client, &args).await?;
            }
        },
        Commands::Label { command } => match command {
            commands::label::LabelCommands::List(args) => {
                commands::label::handle_list(&client, &args).await?;
            }
        },
        Commands::Workflow { command } => match command {
            commands::workflow::WorkflowCommands::List(args) => {
                commands::workflow::handle_list(&client, &args).await?;
            }
        },
        Commands::Comment { command } => match command {
            commands::comment::CommentCommands::List(args) => {
                commands::comment::handle_list(&client, &args).await?;
            }
            commands::comment::CommentCommands::Create(args) => {
                commands::comment::handle_create(&client, &args).await?;
            }
        },
        Commands::Document { command } => match command {
            commands::document::DocumentCommands::List(args) => {
                commands::document::handle_list(&client, &args).await?;
            }
            commands::document::DocumentCommands::Get(args) => {
                commands::document::handle_get(&client, &args).await?;
            }
            commands::document::DocumentCommands::Create(args) => {
                commands::document::handle_create(&client, &args).await?;
            }
        },
        Commands::Search(args) => {
            commands::search::handle_search(&client, &args).await?;
        }
    }

    Ok(())
}
