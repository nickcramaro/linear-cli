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
