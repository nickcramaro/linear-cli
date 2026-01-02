use clap::Args;
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Maximum results
    #[arg(short = 'n', long, default_value = "10")]
    pub limit: u32,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(rename = "searchIssues")]
    search_issues: SearchConnection,
}

#[derive(Deserialize)]
struct SearchConnection {
    nodes: Vec<SearchResult>,
}

#[derive(Deserialize, Clone)]
pub struct SearchResult {
    #[allow(dead_code)]
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub state: Option<SearchState>,
}

#[derive(Deserialize, Clone)]
pub struct SearchState {
    pub name: String,
}

const SEARCH_QUERY: &str = r#"
    query SearchIssues($query: String!, $first: Int) {
        searchIssues(query: $query, first: $first) {
            nodes {
                id
                identifier
                title
                state {
                    name
                }
            }
        }
    }
"#;

pub async fn handle_search(client: &LinearClient, args: &SearchArgs) -> Result<()> {
    let variables = serde_json::json!({
        "query": args.query,
        "first": args.limit
    });

    let response: SearchResponse = client.query(SEARCH_QUERY, variables).await?;
    output::print_search_results(&response.search_issues.nodes);
    Ok(())
}
