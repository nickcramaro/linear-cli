pub mod issue;
pub mod user;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// User operations
    User {
        #[command(subcommand)]
        command: user::UserCommands,
    },
    /// Issue operations
    Issue {
        #[command(subcommand)]
        command: issue::IssueCommands,
    },
}
