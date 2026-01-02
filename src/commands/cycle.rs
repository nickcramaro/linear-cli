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

#[derive(Deserialize)]
struct CyclesResponse {
    cycles: CyclesConnection,
}

#[derive(Deserialize)]
struct CyclesConnection {
    nodes: Vec<Cycle>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
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

#[derive(Deserialize)]
struct CycleResponse {
    cycle: CycleDetail,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CycleDetail {
    pub id: String,
    pub number: i32,
    pub name: Option<String>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: String,
    pub progress: f64,
    pub description: Option<String>,
}

const CYCLES_QUERY: &str = r#"
    query Cycles($first: Int, $filter: CycleFilter) {
        cycles(first: $first, filter: $filter) {
            nodes {
                id
                number
                name
                startsAt
                endsAt
                progress
            }
        }
    }
"#;

const CYCLE_QUERY: &str = r#"
    query Cycle($id: String!) {
        cycle(id: $id) {
            id
            number
            name
            startsAt
            endsAt
            progress
            description
        }
    }
"#;

pub async fn handle_list(client: &LinearClient, args: &ListCycleArgs) -> Result<()> {
    let mut filter = serde_json::Map::new();
    if let Some(team) = &args.team {
        filter.insert(
            "team".to_string(),
            serde_json::json!({ "key": { "eq": team } }),
        );
    }

    let variables = serde_json::json!({
        "first": args.limit,
        "filter": filter
    });

    let response: CyclesResponse = client.query(CYCLES_QUERY, variables).await?;
    output::print_cycles(&response.cycles.nodes);
    Ok(())
}

pub async fn handle_get(client: &LinearClient, args: &GetCycleArgs) -> Result<()> {
    let variables = serde_json::json!({ "id": args.id });
    let response: CycleResponse = client.query(CYCLE_QUERY, variables).await?;
    output::print_cycle_detail(&response.cycle);
    Ok(())
}
