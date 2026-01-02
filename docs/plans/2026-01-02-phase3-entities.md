# Phase 3: Supporting Entities Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add team, project, cycle, label, and workflow commands.

**Architecture:** Follow established patterns from Phase 2. Each entity gets list/get commands. Projects also get create. Workflow lists states per team.

**Tech Stack:** Existing Rust stack

---

### Task 1: Add Team Commands

**Files:**
- Create: `src/commands/team.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Create src/commands/team.rs**

```rust
use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum TeamCommands {
    /// List all teams
    List,
    /// Get team details
    Get(GetTeamArgs),
}

#[derive(Args)]
pub struct GetTeamArgs {
    /// Team key or ID (e.g., ENG)
    pub key: String,
}

#[derive(Deserialize)]
struct TeamsResponse {
    teams: TeamsConnection,
}

#[derive(Deserialize)]
struct TeamsConnection {
    nodes: Vec<Team>,
}

#[derive(Deserialize, Clone)]
pub struct Team {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
struct TeamResponse {
    team: Team,
}

const TEAMS_QUERY: &str = r#"
    query Teams {
        teams {
            nodes {
                id
                key
                name
                description
            }
        }
    }
"#;

const TEAM_QUERY: &str = r#"
    query Team($id: String!) {
        team(id: $id) {
            id
            key
            name
            description
        }
    }
"#;

pub async fn handle_list(client: &LinearClient) -> Result<()> {
    let response: TeamsResponse = client.query(TEAMS_QUERY, serde_json::json!({})).await?;
    output::print_teams(&response.teams.nodes);
    Ok(())
}

pub async fn handle_get(client: &LinearClient, args: &GetTeamArgs) -> Result<()> {
    let variables = serde_json::json!({ "id": args.key });
    let response: TeamResponse = client.query(TEAM_QUERY, variables).await?;
    output::print_team_detail(&response.team);
    Ok(())
}
```

**Step 2: Add to src/commands/mod.rs**

```rust
pub mod issue;
pub mod team;
pub mod user;
```

Add to Commands enum:
```rust
/// Team operations
Team {
    #[command(subcommand)]
    command: team::TeamCommands,
},
```

**Step 3: Add to src/output.rs**

```rust
use crate::commands::team::Team;

pub fn print_teams(teams: &[Team]) {
    if teams.is_empty() {
        println!("No teams found.");
        return;
    }

    for team in teams {
        println!("{} - {}",
            team.key.if_supports_color(Stream::Stdout, |s| s.cyan().bold()),
            team.name);
    }
}

pub fn print_team_detail(team: &Team) {
    println!("{} {}",
        team.key.if_supports_color(Stream::Stdout, |s| s.cyan().bold()),
        team.name.if_supports_color(Stream::Stdout, |s| s.bold()));

    if let Some(desc) = &team.description {
        if !desc.is_empty() {
            println!();
            println!("{}", desc);
        }
    }
}
```

**Step 4: Wire up in src/main.rs**

```rust
Commands::Team { command } => match command {
    commands::team::TeamCommands::List => {
        commands::team::handle_list(&client).await?;
    }
    commands::team::TeamCommands::Get(args) => {
        commands::team::handle_get(&client, &args).await?;
    }
},
```

**Step 5: Verify and Commit**

```bash
cargo build
git add -A
git commit -m "feat: add team list and get commands"
```

---

### Task 2: Add Project Commands

**Files:**
- Create: `src/commands/project.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Create src/commands/project.rs**

```rust
use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List projects
    List(ListProjectArgs),
    /// Get project details
    Get(GetProjectArgs),
    /// Create a new project
    Create(CreateProjectArgs),
}

#[derive(Args)]
pub struct ListProjectArgs {
    /// Filter by team key
    #[arg(long)]
    pub team: Option<String>,

    /// Maximum number to show
    #[arg(short = 'n', long, default_value = "25")]
    pub limit: u32,
}

#[derive(Args)]
pub struct GetProjectArgs {
    /// Project ID or name
    pub id: String,
}

#[derive(Args)]
pub struct CreateProjectArgs {
    /// Project name
    #[arg(long)]
    pub name: String,

    /// Team key
    #[arg(long)]
    pub team: String,

    /// Project description
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
struct ProjectsResponse {
    projects: ProjectsConnection,
}

#[derive(Deserialize)]
struct ProjectsConnection {
    nodes: Vec<Project>,
}

#[derive(Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub state: String,
    pub progress: f64,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "targetDate")]
    pub target_date: Option<String>,
}

#[derive(Deserialize)]
struct ProjectResponse {
    project: ProjectDetail,
}

#[derive(Deserialize)]
pub struct ProjectDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub progress: f64,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "targetDate")]
    pub target_date: Option<String>,
}

#[derive(Deserialize)]
struct CreateProjectResponse {
    #[serde(rename = "projectCreate")]
    project_create: ProjectCreatePayload,
}

#[derive(Deserialize)]
struct ProjectCreatePayload {
    success: bool,
    project: Option<CreatedProject>,
}

#[derive(Deserialize)]
struct CreatedProject {
    id: String,
    name: String,
    url: String,
}

const PROJECTS_QUERY: &str = r#"
    query Projects($first: Int, $filter: ProjectFilter) {
        projects(first: $first, filter: $filter) {
            nodes {
                id
                name
                state
                progress
                startDate
                targetDate
            }
        }
    }
"#;

const PROJECT_QUERY: &str = r#"
    query Project($id: String!) {
        project(id: $id) {
            id
            name
            description
            state
            progress
            startDate
            targetDate
        }
    }
"#;

const CREATE_PROJECT_MUTATION: &str = r#"
    mutation CreateProject($input: ProjectCreateInput!) {
        projectCreate(input: $input) {
            success
            project {
                id
                name
                url
            }
        }
    }
"#;

pub async fn handle_list(client: &LinearClient, args: &ListProjectArgs) -> Result<()> {
    let mut filter = serde_json::Map::new();
    if let Some(team) = &args.team {
        filter.insert("accessibleTeams".to_string(),
            serde_json::json!({ "key": { "eq": team } }));
    }

    let variables = serde_json::json!({
        "first": args.limit,
        "filter": filter
    });

    let response: ProjectsResponse = client.query(PROJECTS_QUERY, variables).await?;
    output::print_projects(&response.projects.nodes);
    Ok(())
}

pub async fn handle_get(client: &LinearClient, args: &GetProjectArgs) -> Result<()> {
    let variables = serde_json::json!({ "id": args.id });
    let response: ProjectResponse = client.query(PROJECT_QUERY, variables).await?;
    output::print_project_detail(&response.project);
    Ok(())
}

pub async fn handle_create(client: &LinearClient, args: &CreateProjectArgs) -> Result<()> {
    let mut input = serde_json::Map::new();
    input.insert("name".to_string(), serde_json::json!(args.name));
    input.insert("teamIds".to_string(), serde_json::json!([args.team]));

    if let Some(desc) = &args.description {
        input.insert("description".to_string(), serde_json::json!(desc));
    }

    let variables = serde_json::json!({ "input": input });
    let response: CreateProjectResponse = client.query(CREATE_PROJECT_MUTATION, variables).await?;

    if response.project_create.success {
        if let Some(project) = response.project_create.project {
            println!("Created project: {}", project.name);
            println!("{}", project.url);
        }
    } else {
        return Err(crate::error::Error::GraphQL("Failed to create project".to_string()));
    }

    Ok(())
}
```

**Step 2: Add to mod.rs and output.rs** (similar pattern to team)

**Step 3: Wire up in main.rs**

**Step 4: Commit**

```bash
git commit -m "feat: add project list, get, and create commands"
```

---

### Task 3: Add Cycle Commands

**Files:**
- Create: `src/commands/cycle.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Create src/commands/cycle.rs**

```rust
use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum CycleCommands {
    /// List cycles
    List(ListCycleArgs),
    /// Get cycle details
    Get(GetCycleArgs),
}

#[derive(Args)]
pub struct ListCycleArgs {
    /// Filter by team key
    #[arg(long)]
    pub team: Option<String>,

    /// Maximum number to show
    #[arg(short = 'n', long, default_value = "10")]
    pub limit: u32,
}

#[derive(Args)]
pub struct GetCycleArgs {
    /// Cycle ID
    pub id: String,
}

#[derive(Deserialize, Clone)]
pub struct Cycle {
    pub id: String,
    pub number: i32,
    pub name: Option<String>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: String,
    pub progress: f64,
}

// Similar pattern for queries and handlers...
```

**Step 2-4:** Follow established patterns, commit as "feat: add cycle list and get commands"

---

### Task 4: Add Label Commands

**Files:**
- Create: `src/commands/label.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Create src/commands/label.rs**

```rust
use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum LabelCommands {
    /// List labels
    List(ListLabelArgs),
}

#[derive(Args)]
pub struct ListLabelArgs {
    /// Filter by team key
    #[arg(long)]
    pub team: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
}

// Query and handler...
```

**Commit:** "feat: add label list command"

---

### Task 5: Add Workflow Commands

**Files:**
- Create: `src/commands/workflow.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Create src/commands/workflow.rs**

```rust
use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum WorkflowCommands {
    /// List workflow states for a team
    List(ListWorkflowArgs),
}

#[derive(Args)]
pub struct ListWorkflowArgs {
    /// Team key (required)
    #[arg(long)]
    pub team: String,
}

#[derive(Deserialize, Clone)]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
    pub color: String,
    pub position: f64,
}

const WORKFLOW_STATES_QUERY: &str = r#"
    query WorkflowStates($teamId: String!) {
        team(id: $teamId) {
            states {
                nodes {
                    id
                    name
                    type
                    color
                    position
                }
            }
        }
    }
"#;

pub async fn handle_list(client: &LinearClient, args: &ListWorkflowArgs) -> Result<()> {
    let variables = serde_json::json!({ "teamId": args.team });
    // Query and print states grouped by type (backlog, unstarted, started, completed, canceled)
    Ok(())
}
```

**Commit:** "feat: add workflow list command"

---

### Task 6: Final Verification and Cleanup

**Step 1:** Run clippy and fmt
**Step 2:** Build release
**Step 3:** Verify all new commands show help
**Step 4:** Commit cleanup

---

## Summary

After completing all tasks:

- `linear team list` / `linear team get <key>`
- `linear project list` / `linear project get <id>` / `linear project create`
- `linear cycle list` / `linear cycle get <id>`
- `linear label list`
- `linear workflow list --team <key>`

**Next Phase:** Phase 4 - Comments, documents, and search.
