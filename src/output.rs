use owo_colors::OwoColorize;

pub fn print_user(name: &str, email: &str, id: &str) {
    println!("{}: {}", "Name".bold(), name);
    println!("{}: {}", "Email".bold(), email);
    println!("{}: {}", "ID".bold().dimmed(), id.dimmed());
}

pub fn print_error(error: &crate::error::Error) {
    eprintln!("{}: {}", "Error".red().bold(), error);
}
