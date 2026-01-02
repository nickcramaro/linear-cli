use owo_colors::{OwoColorize, Stream, Style};
use tabled::{Table, Tabled};
use crate::commands::issue::Issue;

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

    let rows: Vec<IssueRow> = issues.iter().map(|issue| {
        IssueRow {
            id: issue.identifier.clone(),
            title: truncate(&issue.title, 40),
            state: issue.state.as_ref().map(|s| s.name.clone()).unwrap_or_else(|| "-".to_string()),
            assignee: issue.assignee.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| "-".to_string()),
            priority: priority_label(issue.priority),
        }
    }).collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max {
        format!("{}…", chars[..max.saturating_sub(1)].iter().collect::<String>())
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
