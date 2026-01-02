use crate::commands::cycle::{Cycle, CycleDetail};
use crate::commands::issue::{Issue, IssueDetail};
use crate::commands::label::Label;
use crate::commands::project::{Project, ProjectDetail};
use crate::commands::team::Team;
use crate::commands::workflow::WorkflowState;
use owo_colors::{OwoColorize, Stream, Style};
use tabled::{Table, Tabled};

pub fn print_user(name: &str, email: &str, id: &str) {
    println!(
        "{}: {}",
        "Name".if_supports_color(Stream::Stdout, |s| s.bold()),
        name
    );
    println!(
        "{}: {}",
        "Email".if_supports_color(Stream::Stdout, |s| s.bold()),
        email
    );
    let id_style = Style::new().bold().dimmed();
    let dimmed_style = Style::new().dimmed();
    println!(
        "{}: {}",
        "ID".if_supports_color(Stream::Stdout, |s| s.style(id_style)),
        id.if_supports_color(Stream::Stdout, |s| s.style(dimmed_style))
    );
}

pub fn print_error(error: &crate::error::Error) {
    let error_style = Style::new().red().bold();
    eprintln!(
        "{}: {}",
        "Error".if_supports_color(Stream::Stderr, |s| s.style(error_style)),
        error
    );
}

#[derive(Tabled)]
struct IssueRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "Assignee")]
    assignee: String,
    #[tabled(rename = "Priority")]
    priority: String,
}

pub fn print_issues(issues: &[Issue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }

    let rows: Vec<IssueRow> = issues
        .iter()
        .map(|issue| IssueRow {
            id: issue.identifier.clone(),
            title: truncate(&issue.title, 40),
            state: issue
                .state
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "-".to_string()),
            assignee: issue
                .assignee
                .as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "-".to_string()),
            priority: priority_label(issue.priority),
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max {
        format!(
            "{}…",
            chars[..max.saturating_sub(1)].iter().collect::<String>()
        )
    } else {
        s.to_string()
    }
}

fn priority_label(p: i32) -> String {
    match p {
        0 => "None".to_string(),
        1 => "Urgent".to_string(),
        2 => "High".to_string(),
        3 => "Normal".to_string(),
        4 => "Low".to_string(),
        _ => "—".to_string(),
    }
}

pub fn print_issue_detail(issue: &IssueDetail) {
    let id_style = Style::new().cyan().bold();
    let title_style = Style::new().bold();
    println!(
        "{} {}",
        issue
            .identifier
            .if_supports_color(Stream::Stdout, |s| s.style(id_style)),
        issue
            .title
            .if_supports_color(Stream::Stdout, |s| s.style(title_style))
    );
    println!();

    println!(
        "{}: {} ({})",
        "Team".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        issue.team.name,
        issue.team.key
    );
    println!(
        "{}: {}",
        "State".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        issue.state.as_ref().map(|s| s.name.as_str()).unwrap_or("—")
    );
    println!(
        "{}: {}",
        "Assignee".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        issue
            .assignee
            .as_ref()
            .map(|a| a.name.as_str())
            .unwrap_or("—")
    );
    println!(
        "{}: {}",
        "Priority".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        priority_label(issue.priority)
    );
    println!(
        "{}: {}",
        "Created".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        &issue.created_at[..10]
    );
    println!(
        "{}: {}",
        "Updated".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        &issue.updated_at[..10]
    );

    if let Some(desc) = &issue.description {
        if !desc.is_empty() {
            println!();
            println!(
                "{}",
                "Description:".if_supports_color(Stream::Stdout, |s| s.dimmed())
            );
            println!("{}", desc);
        }
    }
}

pub fn print_teams(teams: &[Team]) {
    if teams.is_empty() {
        println!("No teams found.");
        return;
    }

    let key_style = Style::new().cyan().bold();
    for team in teams {
        println!(
            "{} - {}",
            team.key
                .if_supports_color(Stream::Stdout, |s| s.style(key_style)),
            team.name
        );
    }
}

pub fn print_team_detail(team: &Team) {
    let key_style = Style::new().cyan().bold();
    let name_style = Style::new().bold();
    println!(
        "{} {}",
        team.key
            .if_supports_color(Stream::Stdout, |s| s.style(key_style)),
        team.name
            .if_supports_color(Stream::Stdout, |s| s.style(name_style))
    );

    if let Some(desc) = &team.description {
        if !desc.is_empty() {
            println!();
            println!("{}", desc);
        }
    }
}

pub fn print_projects(projects: &[Project]) {
    if projects.is_empty() {
        println!("No projects found.");
        return;
    }

    for project in projects {
        let progress = format!("{:.0}%", project.progress * 100.0);
        println!(
            "{} [{}] {}",
            project.name.if_supports_color(Stream::Stdout, |s| s.bold()),
            project.state,
            progress.if_supports_color(Stream::Stdout, |s| s.dimmed())
        );
    }
}

pub fn print_project_detail(project: &ProjectDetail) {
    println!(
        "{}",
        project.name.if_supports_color(Stream::Stdout, |s| s.bold())
    );
    println!();
    println!(
        "{}: {}",
        "State".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        project.state
    );
    println!(
        "{}: {:.0}%",
        "Progress".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        project.progress * 100.0
    );

    if let Some(start) = &project.start_date {
        println!(
            "{}: {}",
            "Start".if_supports_color(Stream::Stdout, |s| s.dimmed()),
            &start[..10]
        );
    }
    if let Some(target) = &project.target_date {
        println!(
            "{}: {}",
            "Target".if_supports_color(Stream::Stdout, |s| s.dimmed()),
            &target[..10]
        );
    }

    if let Some(desc) = &project.description {
        if !desc.is_empty() {
            println!();
            println!("{}", desc);
        }
    }
}

pub fn print_cycles(cycles: &[Cycle]) {
    if cycles.is_empty() {
        println!("No cycles found.");
        return;
    }

    let number_style = Style::new().cyan().bold();
    for cycle in cycles {
        let name = cycle.name.as_deref().unwrap_or("");
        let progress = format!("{:.0}%", cycle.progress * 100.0);
        let dates = format!("{} → {}", &cycle.starts_at[..10], &cycle.ends_at[..10]);
        let number_str = cycle.number.to_string();
        println!(
            "Cycle {} {} {} {}",
            number_str.if_supports_color(Stream::Stdout, |s| s.style(number_style)),
            name,
            dates.if_supports_color(Stream::Stdout, |s| s.dimmed()),
            progress.if_supports_color(Stream::Stdout, |s| s.dimmed())
        );
    }
}

pub fn print_cycle_detail(cycle: &CycleDetail) {
    let number_style = Style::new().cyan().bold();
    let name_style = Style::new().bold();
    let name = cycle.name.as_deref().unwrap_or("");
    let number_str = cycle.number.to_string();
    println!(
        "Cycle {} {}",
        number_str.if_supports_color(Stream::Stdout, |s| s.style(number_style)),
        name.if_supports_color(Stream::Stdout, |s| s.style(name_style))
    );
    println!();
    println!(
        "{}: {} → {}",
        "Period".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        &cycle.starts_at[..10],
        &cycle.ends_at[..10]
    );
    println!(
        "{}: {:.0}%",
        "Progress".if_supports_color(Stream::Stdout, |s| s.dimmed()),
        cycle.progress * 100.0
    );

    if let Some(desc) = &cycle.description {
        if !desc.is_empty() {
            println!();
            println!("{}", desc);
        }
    }
}

pub fn print_labels(labels: &[Label]) {
    if labels.is_empty() {
        println!("No labels found.");
        return;
    }

    for label in labels {
        println!(
            "{} {}",
            "\u{25cf}".if_supports_color(Stream::Stdout, |s| s.style(Style::new())),
            label.name
        );
    }
}

pub fn print_workflow_states(states: &[WorkflowState]) {
    if states.is_empty() {
        println!("No workflow states found.");
        return;
    }

    // Group by state type and display
    let type_order = ["backlog", "unstarted", "started", "completed", "canceled"];

    for state_type in &type_order {
        let matching: Vec<_> = states
            .iter()
            .filter(|s| s.state_type == *state_type)
            .collect();
        if !matching.is_empty() {
            println!(
                "{}:",
                state_type
                    .to_uppercase()
                    .if_supports_color(Stream::Stdout, |s| s.bold())
            );
            for state in matching {
                println!("  {}", state.name);
            }
        }
    }
}
