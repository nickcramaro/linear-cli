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

#[derive(Deserialize)]
struct LabelsResponse {
    #[serde(rename = "issueLabels")]
    issue_labels: LabelsConnection,
}

#[derive(Deserialize)]
struct LabelsConnection {
    nodes: Vec<Label>,
}

#[derive(Deserialize, Clone)]
pub struct Label {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    #[allow(dead_code)]
    pub color: String,
}

const LABELS_QUERY: &str = r#"
    query Labels($filter: IssueLabelFilter) {
        issueLabels(filter: $filter) {
            nodes {
                id
                name
                color
            }
        }
    }
"#;

pub async fn handle_list(client: &LinearClient, args: &ListLabelArgs) -> Result<()> {
    let mut filter = serde_json::Map::new();
    if let Some(team) = &args.team {
        filter.insert(
            "team".to_string(),
            serde_json::json!({ "key": { "eq": team } }),
        );
    }

    let variables = serde_json::json!({ "filter": filter });
    let response: LabelsResponse = client.query(LABELS_QUERY, variables).await?;
    output::print_labels(&response.issue_labels.nodes);
    Ok(())
}
