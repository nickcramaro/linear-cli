use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::LinearClient;
use crate::error::Result;
use crate::output;

#[derive(Subcommand)]
pub enum DocumentCommands {
    /// List documents
    List(ListDocumentArgs),
    /// Get document details
    Get(GetDocumentArgs),
    /// Create a new document
    Create(CreateDocumentArgs),
}

#[derive(Args)]
pub struct ListDocumentArgs {
    /// Filter by project name or ID
    #[arg(long)]
    pub project: Option<String>,

    /// Maximum number to show
    #[arg(short = 'n', long, default_value = "25")]
    pub limit: u32,
}

#[derive(Args)]
pub struct GetDocumentArgs {
    /// Document ID
    pub id: String,
}

#[derive(Args)]
pub struct CreateDocumentArgs {
    /// Document title
    #[arg(long)]
    pub title: String,

    /// Project ID to attach document to
    #[arg(long)]
    pub project: String,

    /// Document content (markdown)
    #[arg(long)]
    pub content: Option<String>,
}

#[derive(Deserialize)]
struct DocumentsResponse {
    documents: DocumentsConnection,
}

#[derive(Deserialize)]
struct DocumentsConnection {
    nodes: Vec<Document>,
}

#[derive(Deserialize, Clone)]
pub struct Document {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Deserialize)]
struct DocumentResponse {
    document: DocumentDetail,
}

#[derive(Deserialize)]
pub struct DocumentDetail {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Deserialize)]
struct CreateDocumentResponse {
    #[serde(rename = "documentCreate")]
    document_create: DocumentCreatePayload,
}

#[derive(Deserialize)]
struct DocumentCreatePayload {
    success: bool,
    document: Option<CreatedDocument>,
}

#[derive(Deserialize)]
struct CreatedDocument {
    id: String,
    title: String,
}

const DOCUMENTS_QUERY: &str = r#"
    query Documents($first: Int, $filter: DocumentFilter) {
        documents(first: $first, filter: $filter) {
            nodes {
                id
                title
                updatedAt
            }
        }
    }
"#;

const DOCUMENT_QUERY: &str = r#"
    query Document($id: String!) {
        document(id: $id) {
            id
            title
            content
            createdAt
            updatedAt
        }
    }
"#;

const CREATE_DOCUMENT_MUTATION: &str = r#"
    mutation CreateDocument($input: DocumentCreateInput!) {
        documentCreate(input: $input) {
            success
            document {
                id
                title
            }
        }
    }
"#;

pub async fn handle_list(client: &LinearClient, args: &ListDocumentArgs) -> Result<()> {
    let mut filter = serde_json::Map::new();
    if let Some(project) = &args.project {
        filter.insert(
            "project".to_string(),
            serde_json::json!({ "id": { "eq": project } }),
        );
    }

    let variables = serde_json::json!({
        "first": args.limit,
        "filter": filter
    });

    let response: DocumentsResponse = client.query(DOCUMENTS_QUERY, variables).await?;
    output::print_documents(&response.documents.nodes);
    Ok(())
}

pub async fn handle_get(client: &LinearClient, args: &GetDocumentArgs) -> Result<()> {
    let variables = serde_json::json!({ "id": args.id });
    let response: DocumentResponse = client.query(DOCUMENT_QUERY, variables).await?;
    output::print_document_detail(&response.document);
    Ok(())
}

pub async fn handle_create(client: &LinearClient, args: &CreateDocumentArgs) -> Result<()> {
    let mut input = serde_json::Map::new();
    input.insert("title".to_string(), serde_json::json!(args.title));
    input.insert("projectId".to_string(), serde_json::json!(args.project));

    if let Some(content) = &args.content {
        input.insert("content".to_string(), serde_json::json!(content));
    }

    let variables = serde_json::json!({ "input": input });
    let response: CreateDocumentResponse =
        client.query(CREATE_DOCUMENT_MUTATION, variables).await?;

    if response.document_create.success {
        if let Some(doc) = response.document_create.document {
            println!("Created document: {}", doc.title);
            println!("ID: {}", doc.id);
        }
    } else {
        return Err(crate::error::Error::GraphQL(
            "Failed to create document".to_string(),
        ));
    }

    Ok(())
}
