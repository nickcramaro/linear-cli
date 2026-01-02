# Phase 2: Core Issue Operations Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add issue management commands: list, get, create, update.

**Architecture:** Extend existing CLI structure with issue subcommand. Use GraphQL queries/mutations with proper filtering and pagination support. Table output for lists, detail view for single issues.

**Tech Stack:** Existing Rust stack (clap, reqwest, tokio, tabled, owo-colors, serde)

---

### Task 1: Add Issue Command Structure

**Files:**
- Create: `src/commands/issue.rs`
- Modify: `src/commands/mod.rs`

**Step 1: Create src/commands/issue.rs with command enum**

```rust
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
```

**Step 2: Update src/commands/mod.rs to add Issue command**

```rust
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
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Compiles (with warnings about unused args)

**Step 4: Commit**

```bash
git add src/commands/issue.rs src/commands/mod.rs
git commit -m "feat: add issue command structure"
```

---

### Task 2: Implement issue list Command

**Files:**
- Modify: `src/commands/issue.rs`
- Modify: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Add GraphQL query and handler to src/commands/issue.rs**

Add to the file:

```rust
use serde::Deserialize;
use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

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
```

**Step 2: Add print_issues to src/output.rs**

```rust
use tabled::{Table, Tabled};
use crate::commands::issue::Issue;

#[derive(Tabled)]
struct IssueRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "Assignee")]
    assignee: String,
    #[tabled(rename = "Priority")]
    priority: String,
}

pub fn print_issues(issues: &[Issue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }

    let rows: Vec<IssueRow> = issues.iter().map(|issue| {
        IssueRow {
            id: issue.identifier.clone(),
            title: truncate(&issue.title, 40),
            state: issue.state.as_ref().map(|s| s.name.clone()).unwrap_or_else(|| "—".to_string()),
            assignee: issue.assignee.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| "—".to_string()),
            priority: priority_label(issue.priority),
        }
    }).collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max-1])
    } else {
        s.to_string()
    }
}

fn priority_label(p: i32) -> String {
    match p {
        1 => "Urgent".to_string(),
        2 => "High".to_string(),
        3 => "Normal".to_string(),
        4 => "Low".to_string(),
        _ => "—".to_string(),
    }
}
```

**Step 3: Wire up in src/main.rs**

Update the match in `run()`:

```rust
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
            commands::issue::IssueCommands::Get(_) => {
                println!("issue get - not implemented yet");
            }
            commands::issue::IssueCommands::Create(_) => {
                println!("issue create - not implemented yet");
            }
            commands::issue::IssueCommands::Update(_) => {
                println!("issue update - not implemented yet");
            }
        },
    }

    Ok(())
}
```

**Step 4: Verify**

Run: `cargo build`
Run: `cargo run -- issue list --help`
Expected: Shows list command with filter options

**Step 5: Commit**

```bash
git add src/commands/issue.rs src/output.rs src/main.rs
git commit -m "feat: implement issue list command"
```

---

### Task 3: Implement issue get Command

**Files:**
- Modify: `src/commands/issue.rs`
- Modify: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Add get query and handler to src/commands/issue.rs**

Add to the file:

```rust
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

pub async fn handle_get(client: &LinearClient, args: &GetArgs) -> Result<()> {
    let variables = serde_json::json!({ "id": args.id });
    let response: IssueResponse = client.query(ISSUE_QUERY, variables).await?;
    output::print_issue_detail(&response.issue);
    Ok(())
}
```

**Step 2: Add print_issue_detail to src/output.rs**

```rust
use crate::commands::issue::IssueDetail;

pub fn print_issue_detail(issue: &IssueDetail) {
    println!("{} {}",
        issue.identifier.if_supports_color(Stream::Stdout, |s| s.cyan().bold()),
        issue.title.if_supports_color(Stream::Stdout, |s| s.bold()));
    println!();

    println!("{}: {}", "Team".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        format!("{} ({})", issue.team.name, issue.team.key));
    println!("{}: {}", "State".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        issue.state.as_ref().map(|s| s.name.as_str()).unwrap_or("—"));
    println!("{}: {}", "Assignee".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        issue.assignee.as_ref().map(|a| a.name.as_str()).unwrap_or("—"));
    println!("{}: {}", "Priority".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        priority_label(issue.priority));
    println!("{}: {}", "Created".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        &issue.created_at[..10]);
    println!("{}: {}", "Updated".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        &issue.updated_at[..10]);

    if let Some(desc) = &issue.description {
        if !desc.is_empty() {
            println!();
            println!("{}", "Description:".if_supports_color(Stream::Stdout, |s| s.dimmed()));
            println!("{}", desc);
        }
    }
}
```

**Step 3: Wire up in src/main.rs**

Update the Issue match:

```rust
commands::issue::IssueCommands::Get(args) => {
    commands::issue::handle_get(&client, &args).await?;
}
```

**Step 4: Verify and Commit**

Run: `cargo build`

```bash
git add src/commands/issue.rs src/output.rs src/main.rs
git commit -m "feat: implement issue get command"
```

---

### Task 4: Implement issue create Command

**Files:**
- Modify: `src/commands/issue.rs`
- Modify: `src/main.rs`

**Step 1: Add create mutation and handler to src/commands/issue.rs**

```rust
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
        return Err(crate::error::Error::GraphQL("Failed to create issue".to_string()));
    }

    Ok(())
}
```

**Step 2: Wire up in src/main.rs**

```rust
commands::issue::IssueCommands::Create(args) => {
    commands::issue::handle_create(&client, &args).await?;
}
```

**Step 3: Verify and Commit**

```bash
git add src/commands/issue.rs src/main.rs
git commit -m "feat: implement issue create command"
```

---

### Task 5: Implement issue update Command

**Files:**
- Modify: `src/commands/issue.rs`
- Modify: `src/main.rs`

**Step 1: Add update mutation and handler to src/commands/issue.rs**

```rust
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
            let state_name = issue.state.map(|s| s.name).unwrap_or_else(|| "—".to_string());
            println!("Updated {} - {} [{}]", issue.identifier, issue.title, state_name);
        }
    } else {
        return Err(crate::error::Error::GraphQL("Failed to update issue".to_string()));
    }

    Ok(())
}
```

**Step 2: Wire up in src/main.rs**

```rust
commands::issue::IssueCommands::Update(args) => {
    commands::issue::handle_update(&client, &args).await?;
}
```

**Step 3: Verify and Commit**

```bash
git add src/commands/issue.rs src/main.rs
git commit -m "feat: implement issue update command"
```

---

### Task 6: Final Verification and Cleanup

**Step 1: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 2: Run fmt**

Run: `cargo fmt`
Expected: Formats code

**Step 3: Build and verify all commands**

Run: `cargo build --release`

Run: `./target/release/linear issue --help`
Run: `./target/release/linear issue list --help`
Run: `./target/release/linear issue get --help`
Run: `./target/release/linear issue create --help`
Run: `./target/release/linear issue update --help`

**Step 4: Commit if needed**

```bash
git add -A
git commit -m "chore: phase 2 cleanup" --allow-empty
```

---

## Summary

After completing all tasks, you will have:

1. `linear issue list` - List issues with filters (--team, --state, --assignee, -n)
2. `linear issue get <ID>` - Show issue details
3. `linear issue create --team X --title Y` - Create new issues
4. `linear issue update <ID> --state X` - Update issues

**Next Phase:** Phase 3 will add remaining entities (project, team, cycle, label, workflow).
