use clap::{Args, Subcommand};
use serde::Deserialize;
use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

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

#[derive(Deserialize)]
struct IssuesResponse {
    issues: IssuesConnection,
}

#[derive(Deserialize)]
struct IssuesConnection {
    nodes: Vec<Issue>,
}

#[derive(Deserialize)]
pub struct Issue {
    pub identifier: String,
    pub title: String,
    pub state: Option<IssueState>,
    pub assignee: Option<Assignee>,
    pub priority: i32,
}

#[derive(Deserialize)]
pub struct IssueState {
    pub name: String,
}

#[derive(Deserialize)]
pub struct Assignee {
    pub name: String,
}

#[derive(Deserialize)]
struct IssueResponse {
    issue: IssueDetail,
}

#[derive(Deserialize)]
pub struct IssueDetail {
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub state: Option<IssueState>,
    pub assignee: Option<Assignee>,
    pub priority: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub team: Team,
}

#[derive(Deserialize)]
pub struct Team {
    pub key: String,
    pub name: String,
}

const ISSUES_QUERY: &str = r#"
    query Issues($first: Int, $filter: IssueFilter) {
        issues(first: $first, filter: $filter) {
            nodes {
                identifier
                title
                state { name }
                assignee { name }
                priority
            }
        }
    }
"#;

const ISSUE_QUERY: &str = r#"
    query Issue($id: String!) {
        issue(id: $id) {
            identifier
            title
            description
            state { name }
            assignee { name }
            priority
            createdAt
            updatedAt
            team { key name }
        }
    }
"#;

pub async fn handle_list(client: &LinearClient, args: &ListArgs) -> Result<()> {
    let filter = build_filter(args);
    let variables = serde_json::json!({
        "first": args.limit,
        "filter": filter
    });

    let response: IssuesResponse = client.query(ISSUES_QUERY, variables).await?;
    output::print_issues(&response.issues.nodes);
    Ok(())
}

pub async fn handle_get(client: &LinearClient, args: &GetArgs) -> Result<()> {
    let variables = serde_json::json!({ "id": args.id });
    let response: IssueResponse = client.query(ISSUE_QUERY, variables).await?;
    output::print_issue_detail(&response.issue);
    Ok(())
}

fn build_filter(args: &ListArgs) -> serde_json::Value {
    let mut filter = serde_json::Map::new();

    if let Some(team) = &args.team {
        filter.insert("team".to_string(), serde_json::json!({ "key": { "eq": team } }));
    }
    if let Some(state) = &args.state {
        filter.insert("state".to_string(), serde_json::json!({ "name": { "eq": state } }));
    }
    if let Some(assignee) = &args.assignee {
        if assignee == "me" {
            filter.insert("assignee".to_string(), serde_json::json!({ "isMe": { "eq": true } }));
        } else {
            filter.insert("assignee".to_string(), serde_json::json!({ "name": { "contains": assignee } }));
        }
    }

    serde_json::Value::Object(filter)
}
