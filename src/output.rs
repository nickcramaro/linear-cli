use owo_colors::{OwoColorize, Stream, Style};

pub fn print_user(name: &str, email: &str, id: &str) {
    println!("{}: {}", "Name".if_supports_color(Stream::Stdout, |s| s.bold()), name);
    println!("{}: {}", "Email".if_supports_color(Stream::Stdout, |s| s.bold()), email);
    let id_style = Style::new().bold().dimmed();
    let dimmed_style = Style::new().dimmed();
    println!("{}: {}",
        "ID".if_supports_color(Stream::Stdout, |s| s.style(id_style)),
        id.if_supports_color(Stream::Stdout, |s| s.style(dimmed_style)));
}

pub fn print_error(error: &crate::error::Error) {
    let error_style = Style::new().red().bold();
    eprintln!("{}: {}",
        "Error".if_supports_color(Stream::Stderr, |s| s.style(error_style)),
        error);
}
