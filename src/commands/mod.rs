pub mod cycle;
pub mod issue;
pub mod project;
pub mod team;
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
    /// Team operations
    Team {
        #[command(subcommand)]
        command: team::TeamCommands,
    },
    /// Project operations
    Project {
        #[command(subcommand)]
        command: project::ProjectCommands,
    },
    /// Cycle operations
    Cycle {
        #[command(subcommand)]
        command: cycle::CycleCommands,
    },
}
