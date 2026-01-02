use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum CommentCommands {
    /// List comments on an issue
    List(ListCommentArgs),
    /// Create a comment on an issue
    Create(CreateCommentArgs),
}

#[derive(Args)]
pub struct ListCommentArgs {
    /// Issue ID (e.g., ENG-123)
    pub issue: String,
}

#[derive(Args)]
pub struct CreateCommentArgs {
    /// Issue ID (e.g., ENG-123)
    #[arg(long)]
    pub issue: String,

    /// Comment body (markdown supported)
    #[arg(long)]
    pub body: String,
}

#[derive(Deserialize)]
struct IssueCommentsResponse {
    issue: IssueWithComments,
}

#[derive(Deserialize)]
struct IssueWithComments {
    comments: CommentsConnection,
}

#[derive(Deserialize)]
struct CommentsConnection {
    nodes: Vec<Comment>,
}

#[derive(Deserialize, Clone)]
pub struct Comment {
    #[allow(dead_code)]
    pub id: String,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub user: Option<CommentUser>,
}

#[derive(Deserialize, Clone)]
pub struct CommentUser {
    pub name: String,
}

#[derive(Deserialize)]
struct CreateCommentResponse {
    #[serde(rename = "commentCreate")]
    comment_create: CommentCreatePayload,
}

#[derive(Deserialize)]
struct CommentCreatePayload {
    success: bool,
}

const ISSUE_COMMENTS_QUERY: &str = r#"
    query IssueComments($id: String!) {
        issue(id: $id) {
            comments {
                nodes {
                    id
                    body
                    createdAt
                    user {
                        name
                    }
                }
            }
        }
    }
"#;

const CREATE_COMMENT_MUTATION: &str = r#"
    mutation CreateComment($input: CommentCreateInput!) {
        commentCreate(input: $input) {
            success
        }
    }
"#;

pub async fn handle_list(client: &LinearClient, args: &ListCommentArgs) -> Result<()> {
    let variables = serde_json::json!({ "id": args.issue });
    let response: IssueCommentsResponse = client.query(ISSUE_COMMENTS_QUERY, variables).await?;
    output::print_comments(&response.issue.comments.nodes);
    Ok(())
}

pub async fn handle_create(client: &LinearClient, args: &CreateCommentArgs) -> Result<()> {
    let input = serde_json::json!({
        "issueId": args.issue,
        "body": args.body
    });
    let variables = serde_json::json!({ "input": input });
    let response: CreateCommentResponse = client.query(CREATE_COMMENT_MUTATION, variables).await?;

    if response.comment_create.success {
        println!("Comment added.");
    } else {
        return Err(crate::error::Error::GraphQL(
            "Failed to create comment".to_string(),
        ));
    }

    Ok(())
}
