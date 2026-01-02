use clap::Subcommand;

#[derive(Subcommand)]
pub enum UserCommands {
    /// Show current authenticated user
    Me,
}
