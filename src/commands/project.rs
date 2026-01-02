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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
        filter.insert(
            "accessibleTeams".to_string(),
            serde_json::json!({ "key": { "eq": team } }),
        );
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
        return Err(crate::error::Error::GraphQL(
            "Failed to create project".to_string(),
        ));
    }

    Ok(())
}
