use crate::client::LinearClient;
use crate::error::Result;
use crate::output;
use clap::{Args, Subcommand};
use serde::Deserialize;

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

#[derive(Deserialize)]
struct CreateIssueResponse {
    #[serde(rename = "issueCreate")]
    issue_create: IssueCreatePayload,
}

#[derive(Deserialize)]
struct IssueCreatePayload {
    success: bool,
    issue: Option<CreatedIssue>,
}

#[derive(Deserialize)]
struct CreatedIssue {
    identifier: String,
    title: String,
    url: String,
}

const CREATE_ISSUE_MUTATION: &str = r#"
    mutation CreateIssue($input: IssueCreateInput!) {
        issueCreate(input: $input) {
            success
            issue {
                identifier
                title
                url
            }
        }
    }
"#;

#[derive(Deserialize)]
struct UpdateIssueResponse {
    #[serde(rename = "issueUpdate")]
    issue_update: IssueUpdatePayload,
}

#[derive(Deserialize)]
struct IssueUpdatePayload {
    success: bool,
    issue: Option<UpdatedIssue>,
}

#[derive(Deserialize)]
struct UpdatedIssue {
    identifier: String,
    title: String,
    state: Option<IssueState>,
}

const UPDATE_ISSUE_MUTATION: &str = r#"
    mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
        issueUpdate(id: $id, input: $input) {
            success
            issue {
                identifier
                title
                state { name }
            }
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

pub async fn handle_create(client: &LinearClient, args: &CreateArgs) -> Result<()> {
    let mut input = serde_json::Map::new();
    input.insert("title".to_string(), serde_json::json!(args.title));
    input.insert("teamId".to_string(), serde_json::json!(args.team));

    if let Some(desc) = &args.description {
        input.insert("description".to_string(), serde_json::json!(desc));
    }
    if let Some(priority) = args.priority {
        input.insert("priority".to_string(), serde_json::json!(priority));
    }

    let variables = serde_json::json!({ "input": input });
    let response: CreateIssueResponse = client.query(CREATE_ISSUE_MUTATION, variables).await?;

    if response.issue_create.success {
        if let Some(issue) = response.issue_create.issue {
            println!("Created {} - {}", issue.identifier, issue.title);
            println!("{}", issue.url);
        }
    } else {
        return Err(crate::error::Error::GraphQL(
            "Failed to create issue".to_string(),
        ));
    }

    Ok(())
}

pub async fn handle_update(client: &LinearClient, args: &UpdateArgs) -> Result<()> {
    let mut input = serde_json::Map::new();

    if let Some(title) = &args.title {
        input.insert("title".to_string(), serde_json::json!(title));
    }
    if let Some(state) = &args.state {
        input.insert("stateId".to_string(), serde_json::json!(state));
    }
    if let Some(priority) = args.priority {
        input.insert("priority".to_string(), serde_json::json!(priority));
    }

    if input.is_empty() {
        println!("No updates specified. Use --title, --state, or --priority.");
        return Ok(());
    }

    let variables = serde_json::json!({
        "id": args.id,
        "input": input
    });

    let response: UpdateIssueResponse = client.query(UPDATE_ISSUE_MUTATION, variables).await?;

    if response.issue_update.success {
        if let Some(issue) = response.issue_update.issue {
            let state_name = issue
                .state
                .map(|s| s.name)
                .unwrap_or_else(|| "—".to_string());
            println!(
                "Updated {} - {} [{}]",
                issue.identifier, issue.title, state_name
            );
        }
    } else {
        return Err(crate::error::Error::GraphQL(
            "Failed to update issue".to_string(),
        ));
    }

    Ok(())
}

fn build_filter(args: &ListArgs) -> serde_json::Value {
    let mut filter = serde_json::Map::new();

    if let Some(team) = &args.team {
        filter.insert(
            "team".to_string(),
            serde_json::json!({ "key": { "eq": team } }),
        );
    }
    if let Some(state) = &args.state {
        filter.insert(
            "state".to_string(),
            serde_json::json!({ "name": { "eq": state } }),
        );
    }
    if let Some(assignee) = &args.assignee {
        if assignee == "me" {
            filter.insert(
                "assignee".to_string(),
                serde_json::json!({ "isMe": { "eq": true } }),
            );
        } else {
            filter.insert(
                "assignee".to_string(),
                serde_json::json!({ "name": { "contains": assignee } }),
            );
        }
    }

    serde_json::Value::Object(filter)
}
