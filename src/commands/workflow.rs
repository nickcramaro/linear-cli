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

#[derive(Deserialize)]
struct TeamResponse {
    team: TeamWithStates,
}

#[derive(Deserialize)]
struct TeamWithStates {
    states: StatesConnection,
}

#[derive(Deserialize)]
struct StatesConnection {
    nodes: Vec<WorkflowState>,
}

#[derive(Deserialize, Clone)]
pub struct WorkflowState {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
    #[allow(dead_code)]
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
    let response: TeamResponse = client.query(WORKFLOW_STATES_QUERY, variables).await?;

    // Sort states by position for display
    let mut states = response.team.states.nodes;
    states.sort_by(|a, b| {
        a.position
            .partial_cmp(&b.position)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    output::print_workflow_states(&states);
    Ok(())
}
