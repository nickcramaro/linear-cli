use clap::Subcommand;
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum UserCommands {
    /// Show current authenticated user
    Me,
}

#[derive(Deserialize)]
struct ViewerResponse {
    viewer: Viewer,
}

#[derive(Deserialize)]
struct Viewer {
    id: String,
    name: String,
    email: String,
}

const VIEWER_QUERY: &str = r#"
    query Viewer {
        viewer {
            id
            name
            email
        }
    }
"#;

pub async fn handle_me(client: &LinearClient) -> Result<()> {
    let response: ViewerResponse = client.query(VIEWER_QUERY, serde_json::json!({})).await?;
    output::print_user(&response.viewer.name, &response.viewer.email, &response.viewer.id);
    Ok(())
}
