use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum IssueCommands {
    /// List issues
    List(ListArgs),
    /// Get issue details
    Get(GetArgs),
    /// Create a new issue
    Create(CreateArgs),
    /// Update an issue
    Update(UpdateArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter by team key (e.g., ENG)
    #[arg(long)]
    pub team: Option<String>,

    /// Filter by state name (e.g., "In Progress")
    #[arg(long)]
    pub state: Option<String>,

    /// Filter by assignee (use "me" for yourself)
    #[arg(long)]
    pub assignee: Option<String>,

    /// Maximum number of issues to show
    #[arg(short = 'n', long, default_value = "25")]
    pub limit: u32,
}

#[derive(Args)]
pub struct GetArgs {
    /// Issue identifier (e.g., ENG-123)
    pub id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Issue title
    #[arg(long)]
    pub title: String,

    /// Team key (e.g., ENG)
    #[arg(long)]
    pub team: String,

    /// Issue description (markdown)
    #[arg(long)]
    pub description: Option<String>,

    /// Priority (1=urgent, 2=high, 3=normal, 4=low)
    #[arg(long)]
    pub priority: Option<i32>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Issue identifier (e.g., ENG-123)
    pub id: String,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New state name (e.g., "Done")
    #[arg(long)]
    pub state: Option<String>,

    /// New priority (1=urgent, 2=high, 3=normal, 4=low)
    #[arg(long)]
    pub priority: Option<i32>,
}
