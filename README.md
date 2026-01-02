# linear-cli

A fast, native CLI for [Linear](https://linear.app) built in Rust.

## Installation

### Quick Install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/nickcramaro/linear-cli/main/install.sh | sh
```

This automatically detects your OS and architecture.

### Build from Source

```bash
git clone https://github.com/nickcramaro/linear-cli.git
cd linear-cli
cargo build --release
cp target/release/linear /usr/local/bin/
```

## Authentication

Set your Linear API key as an environment variable:

```bash
export LINEAR_API_KEY="lin_api_xxxxx"
```

Get your API key from [Linear Settings > API](https://linear.app/settings/api).

## Commands

### User

```bash
linear user me                    # Show current authenticated user
```

### Issues

```bash
linear issue list                 # List your assigned issues
linear issue list --team ENG      # List issues for a team
linear issue list --state "In Progress"
linear issue list -n 50           # Limit results

linear issue get ENG-123          # Get issue details

linear issue create --team ENG --title "Fix bug"
linear issue create --team ENG --title "Task" --description "Details here"

linear issue update ENG-123 --state Done
linear issue update ENG-123 --assignee me
```

### Teams

```bash
linear team list                  # List all teams
linear team get ENG               # Get team details
```

### Projects

```bash
linear project list               # List all projects
linear project list --team ENG    # Filter by team
linear project get <id>           # Get project details
linear project create --name "Q1 Launch" --team ENG
```

### Cycles

```bash
linear cycle list                 # List cycles
linear cycle list --team ENG      # Filter by team
linear cycle get <id>             # Get cycle details
```

### Labels

```bash
linear label list                 # List all labels
linear label list --team ENG      # Filter by team
```

### Workflow States

```bash
linear workflow list --team ENG   # List workflow states for a team
```

### Comments

```bash
linear comment list ENG-123       # List comments on an issue
linear comment create --issue ENG-123 --body "My comment"
```

### Documents

```bash
linear document list              # List documents
linear document list --project <id>
linear document get <id>          # Get document details
linear document create --title "Doc" --project <id>
```

### Search

```bash
linear search "bug fix"           # Search issues
linear search "login" -n 20       # With custom limit
```

### Update

```bash
linear update                     # Update to the latest version
```

## Options

All commands support:

- `--no-color` - Disable colored output
- `-h, --help` - Show help

The `NO_COLOR` environment variable is also respected.

## License

MIT
