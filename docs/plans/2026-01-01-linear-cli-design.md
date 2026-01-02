# Linear CLI Design Document

A Rust CLI that wraps Linear's GraphQL API with full coverage.

## Goals

- **Self-maintained**: No reliance on third-party CLI tools
- **Full API coverage**: Wrap everything Linear exposes
- **Ergonomic**: Human-readable output, intuitive command structure
- **Fast**: Compiled Rust binary, minimal startup time

## Project Structure

```
linear-cli/
├── Cargo.toml
├── build.rs                    # Schema fetching + codegen trigger
├── schema.graphql              # Linear's schema (fetched/cached)
├── src/
│   ├── main.rs                 # Entry point, clap setup
│   ├── client.rs               # GraphQL client wrapper
│   ├── auth.rs                 # LINEAR_API_KEY handling
│   ├── output.rs               # Table formatting, colors
│   ├── error.rs                # Error types
│   ├── generated/              # Cynic-generated types (don't edit)
│   │   └── mod.rs
│   └── commands/               # Hand-written command implementations
│       ├── mod.rs
│       ├── issue.rs
│       ├── project.rs
│       ├── team.rs
│       ├── cycle.rs
│       ├── user.rs
│       ├── comment.rs
│       ├── document.rs
│       └── label.rs
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI parsing with derive macros |
| `cynic` | GraphQL client with schema-first codegen |
| `reqwest` | HTTP client (with rustls) |
| `tokio` | Async runtime |
| `tabled` | Terminal table formatting |
| `owo-colors` or `colored` | Terminal colors |
| `thiserror` | Error handling |
| `serde` / `serde_json` | Serialization |

## Authentication

Environment variable only: `LINEAR_API_KEY`

```bash
export LINEAR_API_KEY="lin_api_xxxxx"
linear issue list
```

No config file. Simple, secure, works with shell dotfiles and `direnv`.

## Command Structure

**Pattern**: Noun-verb (`linear <entity> <action>`)

### Entity Commands

| Entity | Commands |
|--------|----------|
| `issue` | `list`, `get`, `create`, `update`, `delete`, `search` |
| `project` | `list`, `get`, `create`, `update`, `delete` |
| `team` | `list`, `get` |
| `cycle` | `list`, `get`, `create`, `update` |
| `user` | `list`, `get`, `me` |
| `comment` | `list`, `create`, `update`, `delete` |
| `document` | `list`, `get`, `create`, `update`, `delete` |
| `label` | `list`, `get`, `create` |
| `workflow` | `list` (workflow states per team) |

### Example Usage

```bash
# Issues
linear issue list                      # Your assigned issues
linear issue list --team ENG           # Team's issues
linear issue list --state "In Progress"
linear issue get ABC-123
linear issue create --team ENG --title "Fix bug"
linear issue update ABC-123 --state Done

# Projects
linear project list
linear project get "Q1 Launch"

# Quick info
linear user me                         # Current authenticated user
linear team list
```

### Common Flags

- `--limit` / `-n` - Pagination limit
- `--json` - JSON output (for scripting)
- `--no-color` - Disable colored output
- `--help` - Per-command help

## GraphQL Layer

### Schema Management

- Fetch Linear's schema via introspection at build time (`build.rs`)
- Cache `schema.graphql` in repo (committed)
- Manual refresh: `cargo run --bin fetch-schema`

### Cynic Code Generation

```rust
// src/generated/mod.rs (generated, don't edit)
#[cynic::schema("linear")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Issue")]
pub struct Issue {
    pub id: cynic::Id,
    pub identifier: String,      // ABC-123
    pub title: String,
    pub state: Option<WorkflowState>,
    pub assignee: Option<User>,
    pub priority: i32,
    pub created_at: DateTime,
}
```

### Client Wrapper

```rust
pub struct LinearClient {
    http: reqwest::Client,
    api_key: String,
}

impl LinearClient {
    pub fn from_env() -> Result<Self>;
    pub async fn query<T: cynic::QueryFragment>(&self, query: T) -> Result<T>;

    // High-level methods
    pub async fn list_issues(&self, filter: IssueFilter) -> Result<Vec<Issue>>;
    pub async fn get_issue(&self, id: &str) -> Result<Issue>;
    pub async fn create_issue(&self, input: CreateIssueInput) -> Result<Issue>;
}
```

### Pagination

- Linear uses Relay cursor pagination (`first`, `after`, `last`, `before`)
- Client abstracts this internally
- Commands expose simple `--limit` flag

## Error Handling

```rust
#[derive(thiserror::Error, Debug)]
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
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Auth error (missing/invalid key) |
| 3 | Not found |
| 4 | Rate limited |

## Output Formatting

### Tables

```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
pub struct IssueRow {
    #[tabled(rename = "ID")]
    pub identifier: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "State")]
    pub state: String,
    #[tabled(rename = "Assignee")]
    pub assignee: String,
    #[tabled(rename = "Priority")]
    pub priority: String,
}
```

### Colors

- Respect `NO_COLOR` env var and `--no-color` flag
- **State**: green (Done), yellow (In Progress), gray (Todo/Backlog), red (Canceled)
- **Priority**: red (Urgent), orange (High), white (Normal), dim (Low)
- **Issue IDs**: cyan
- **Errors**: red
- **Success**: green

### Example Output

```
$ linear issue list --limit 3
┌─────────┬─────────────────────┬─────────────┬──────────┬──────────┐
│ ID      │ Title               │ State       │ Assignee │ Priority │
├─────────┼─────────────────────┼─────────────┼──────────┼──────────┤
│ ENG-142 │ Fix login redirect  │ In Progress │ nick     │ High     │
│ ENG-141 │ Add dark mode       │ Todo        │ nick     │ Medium   │
│ ENG-140 │ Update dependencies │ Done        │ —        │ Low      │
└─────────┴─────────────────────┴─────────────┴──────────┴──────────┘
```

## Implementation Phases

### Phase 1: Foundation

- Project setup with `clap`, `cynic`, `reqwest`, `tokio`
- Fetch and cache Linear schema
- `LinearClient` with auth from `LINEAR_API_KEY`
- Basic error types
- `linear user me` as first working command (validates auth)

### Phase 2: Core Issue Operations

- `linear issue list` with filters (`--team`, `--state`, `--assignee`)
- `linear issue get <ID>`
- `linear issue create`
- `linear issue update`
- Table output with colors

### Phase 3: Supporting Entities

- `team list`, `team get`
- `project list`, `project get`, `project create`
- `cycle list`, `cycle get`
- `label list`
- `workflow list`

### Phase 4: Full CRUD

- Remaining create/update/delete operations
- `comment` commands
- `document` commands
- `search` command (cross-entity)

### Phase 5: Polish

- `--json` flag for all commands
- Better error messages with suggestions
- Shell completions (`clap_complete`)
- Man page generation

### Future (Optional)

- Git integration (`issue start`, branch creation)
- Interactive mode / TUI
- Caching for faster repeated queries

## Linear API Reference

- **Endpoint**: `https://api.linear.app/graphql`
- **Auth header**: `Authorization: <API_KEY>`
- **Schema explorer**: Apollo Studio (public)
- **Rate limiting**: Varies, includes retry-after header
- **Pagination**: Relay cursor-based (`first`/`after`, `last`/`before`)
