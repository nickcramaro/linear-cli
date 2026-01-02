# Phase 1: Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up project foundation with working `linear user me` command that validates auth.

**Architecture:** Rust CLI using clap for args, cynic for GraphQL codegen, reqwest for HTTP. Auth via LINEAR_API_KEY env var. Human-readable table output with colors.

**Tech Stack:** Rust, clap (derive), cynic, reqwest (rustls-tls), tokio, tabled, owo-colors, thiserror

---

### Task 1: Initialize Cargo Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Create Cargo.toml with dependencies**

```toml
[package]
name = "linear-cli"
version = "0.1.0"
edition = "2021"
description = "A CLI for Linear"
license = "MIT"

[[bin]]
name = "linear"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
cynic = "3"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tabled = "0.17"
owo-colors = "4"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
cynic-codegen = "3"
```

**Step 2: Create minimal main.rs**

```rust
fn main() {
    println!("linear-cli");
}
```

**Step 3: Verify project builds**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "chore: initialize cargo project with dependencies"
```

---

### Task 2: Fetch Linear GraphQL Schema

**Files:**
- Create: `scripts/fetch-schema.sh`
- Create: `schema.graphql`

**Step 1: Create schema fetch script**

```bash
#!/bin/bash
set -e

SCHEMA_URL="https://api.linear.app/graphql"

# Introspection query
QUERY='{"query":"query IntrospectionQuery { __schema { queryType { name } mutationType { name } subscriptionType { name } types { ...FullType } directives { name description locations args { ...InputValue } } } } fragment FullType on __Type { kind name description fields(includeDeprecated: true) { name description args { ...InputValue } type { ...TypeRef } isDeprecated deprecationReason } inputFields { ...InputValue } interfaces { ...TypeRef } enumValues(includeDeprecated: true) { name description isDeprecated deprecationReason } possibleTypes { ...TypeRef } } fragment InputValue on __InputValue { name description type { ...TypeRef } defaultValue } fragment TypeRef on __Type { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name } } } } } } } }"}'

echo "Fetching Linear GraphQL schema..."
curl -s -X POST "$SCHEMA_URL" \
  -H "Content-Type: application/json" \
  -d "$QUERY" | jq -r '.data' > schema.json

echo "Schema saved to schema.json"
echo "Note: cynic uses JSON introspection format"
```

**Step 2: Make script executable and run it**

Run: `chmod +x scripts/fetch-schema.sh && mkdir -p scripts && mv scripts/fetch-schema.sh scripts/ 2>/dev/null; scripts/fetch-schema.sh`
Expected: Creates `schema.json` with Linear's schema

**Step 3: Verify schema was fetched**

Run: `head -c 200 schema.json`
Expected: JSON starting with `{"queryType":...` or `{"__schema":...`

**Step 4: Commit**

```bash
git add scripts/fetch-schema.sh schema.json
git commit -m "chore: add schema fetch script and Linear schema"
```

---

### Task 3: Set Up Cynic Code Generation

**Files:**
- Create: `build.rs`
- Create: `src/generated.rs`
- Modify: `src/main.rs`

**Step 1: Create build.rs for cynic codegen registration**

```rust
fn main() {
    println!("cargo::rerun-if-changed=schema.json");
}
```

**Step 2: Create src/generated.rs with schema module**

```rust
#[cynic::schema("linear")]
mod schema {}
```

**Step 3: Create .graphqlrc.yml for cynic schema registration**

Create file `.graphqlrc.yml`:
```yaml
schema:
  linear: schema.json
```

**Step 4: Update src/main.rs to include generated module**

```rust
mod generated;

fn main() {
    println!("linear-cli");
}
```

**Step 5: Verify build still works**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add build.rs src/generated.rs .graphqlrc.yml src/main.rs
git commit -m "chore: set up cynic schema registration"
```

---

### Task 4: Create Error Types

**Files:**
- Create: `src/error.rs`
- Modify: `src/main.rs`

**Step 1: Create src/error.rs**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("LINEAR_API_KEY environment variable not set")]
    MissingApiKey,

    #[error("Authentication failed: invalid API key")]
    Unauthorized,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::MissingApiKey | Error::Unauthorized => 2,
            Error::NotFound(_) => 3,
            Error::RateLimited(_) => 4,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Step 2: Update src/main.rs to include error module**

```rust
mod error;
mod generated;

fn main() {
    println!("linear-cli");
}
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/error.rs src/main.rs
git commit -m "feat: add error types with exit codes"
```

---

### Task 5: Create GraphQL Client

**Files:**
- Create: `src/client.rs`
- Modify: `src/main.rs`

**Step 1: Create src/client.rs**

```rust
use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.linear.app/graphql";

pub struct LinearClient {
    http: reqwest::Client,
}

#[derive(Serialize)]
struct GraphQLRequest<T: Serialize> {
    query: String,
    variables: T,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

impl LinearClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("LINEAR_API_KEY").map_err(|_| Error::MissingApiKey)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&api_key).expect("invalid api key format"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build http client");

        Ok(Self { http })
    }

    pub async fn query<V, T>(&self, query: &str, variables: V) -> Result<T>
    where
        V: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let request = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let response = self.http.post(API_URL).json(&request).send().await?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::Unauthorized);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(Error::RateLimited(retry_after));
        }

        let gql_response: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = gql_response.errors {
            let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            return Err(Error::GraphQL(messages.join(", ")));
        }

        gql_response
            .data
            .ok_or_else(|| Error::GraphQL("no data in response".to_string()))
    }
}
```

**Step 2: Update src/main.rs**

```rust
mod client;
mod error;
mod generated;

fn main() {
    println!("linear-cli");
}
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/client.rs src/main.rs
git commit -m "feat: add GraphQL client with auth"
```

---

### Task 6: Create CLI Structure with Clap

**Files:**
- Create: `src/commands/mod.rs`
- Create: `src/commands/user.rs`
- Modify: `src/main.rs`

**Step 1: Create src/commands/mod.rs**

```rust
pub mod user;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// User operations
    User {
        #[command(subcommand)]
        command: user::UserCommands,
    },
}
```

**Step 2: Create src/commands/user.rs**

```rust
use clap::Subcommand;

#[derive(Subcommand)]
pub enum UserCommands {
    /// Show current authenticated user
    Me,
}
```

**Step 3: Update src/main.rs with clap CLI**

```rust
mod client;
mod commands;
mod error;
mod generated;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::User { command } => match command {
            commands::user::UserCommands::Me => {
                println!("user me - not implemented yet");
            }
        },
    }
}
```

**Step 4: Verify CLI works**

Run: `cargo run -- user me`
Expected: Prints "user me - not implemented yet"

Run: `cargo run -- --help`
Expected: Shows help with "user" subcommand

**Step 5: Commit**

```bash
git add src/commands/mod.rs src/commands/user.rs src/main.rs
git commit -m "feat: add CLI structure with clap"
```

---

### Task 7: Implement `user me` Command

**Files:**
- Modify: `src/commands/user.rs`
- Modify: `src/main.rs`
- Create: `src/output.rs`

**Step 1: Create src/output.rs for formatting**

```rust
use owo_colors::OwoColorize;

pub fn print_user(name: &str, email: &str, id: &str) {
    println!("{}: {}", "Name".bold(), name);
    println!("{}: {}", "Email".bold(), email);
    println!("{}: {}", "ID".bold().dimmed(), id.dimmed());
}

pub fn print_error(error: &crate::error::Error) {
    eprintln!("{}: {}", "Error".red().bold(), error);
}
```

**Step 2: Update src/commands/user.rs to implement me command**

```rust
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
```

**Step 3: Update src/main.rs to run async and handle errors**

```rust
mod client;
mod commands;
mod error;
mod generated;
mod output;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        output::print_error(&e);
        std::process::exit(e.exit_code());
    }
}

async fn run(cli: Cli) -> error::Result<()> {
    let client = client::LinearClient::from_env()?;

    match cli.command {
        Commands::User { command } => match command {
            commands::user::UserCommands::Me => {
                commands::user::handle_me(&client).await?;
            }
        },
    }

    Ok(())
}
```

**Step 4: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 5: Test without API key**

Run: `unset LINEAR_API_KEY && cargo run -- user me`
Expected: Error message about missing LINEAR_API_KEY

**Step 6: Test with API key (manual verification)**

Run: `LINEAR_API_KEY=<your-key> cargo run -- user me`
Expected: Shows your Linear user name, email, and ID

**Step 7: Commit**

```bash
git add src/output.rs src/commands/user.rs src/main.rs
git commit -m "feat: implement user me command"
```

---

### Task 8: Add Color Support Toggle

**Files:**
- Modify: `src/main.rs`
- Modify: `src/output.rs`

**Step 1: Update src/main.rs with --no-color flag**

```rust
mod client;
mod commands;
mod error;
mod generated;
mod output;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle NO_COLOR env var and --no-color flag
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        owo_colors::set_override(false);
    }

    if let Err(e) = run(cli).await {
        output::print_error(&e);
        std::process::exit(e.exit_code());
    }
}

async fn run(cli: Cli) -> error::Result<()> {
    let client = client::LinearClient::from_env()?;

    match cli.command {
        Commands::User { command } => match command {
            commands::user::UserCommands::Me => {
                commands::user::handle_me(&client).await?;
            }
        },
    }

    Ok(())
}
```

**Step 2: Verify --no-color works**

Run: `cargo run -- --no-color user me`
Expected: Output without colors (if you have LINEAR_API_KEY set)

Run: `cargo run -- --help`
Expected: Help shows --no-color flag

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add --no-color flag and NO_COLOR env support"
```

---

### Task 9: Final Verification and Cleanup

**Step 1: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 2: Run fmt**

Run: `cargo fmt`
Expected: No changes (or formats code)

**Step 3: Verify full flow**

Run: `cargo build --release`
Expected: Builds successfully

Run: `./target/release/linear --help`
Expected: Shows help

Run: `./target/release/linear user --help`
Expected: Shows user subcommand help

**Step 4: Final commit if any changes**

```bash
git add -A
git commit -m "chore: clippy and fmt cleanup" --allow-empty
```

---

## Summary

After completing all tasks, you will have:

1. A working Rust CLI project with proper dependencies
2. Linear's GraphQL schema cached locally
3. Cynic set up for future codegen
4. Proper error types with exit codes
5. A GraphQL client with auth from LINEAR_API_KEY
6. Working `linear user me` command with colored output
7. --no-color flag and NO_COLOR env var support

**Next Phase:** Phase 2 will add core issue operations (list, get, create, update).
