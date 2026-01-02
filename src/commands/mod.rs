pub mod comment;
pub mod cycle;
pub mod document;
pub mod issue;
pub mod label;
pub mod project;
pub mod search;
pub mod team;
pub mod user;
pub mod workflow;

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
    /// Label operations
    Label {
        #[command(subcommand)]
        command: label::LabelCommands,
    },
    /// Workflow operations
    Workflow {
        #[command(subcommand)]
        command: workflow::WorkflowCommands,
    },
    /// Comment operations
    Comment {
        #[command(subcommand)]
        command: comment::CommentCommands,
    },
    /// Document operations
    Document {
        #[command(subcommand)]
        command: document::DocumentCommands,
    },
    /// Search issues
    Search(search::SearchArgs),
}
