use colored::{ColoredString, Colorize};

pub fn path(s: &str) -> ColoredString {
    s.cyan()
}

pub fn label(s: &str) -> ColoredString {
    s.bold()
}

pub fn dim(s: &str) -> ColoredString {
    s.bright_black()
}

pub fn success(s: &str) -> ColoredString {
    s.green()
}

pub fn error(s: &str) -> ColoredString {
    s.red()
}

pub fn warn(s: &str) -> ColoredString {
    s.yellow()
}

pub fn kv(key: &str, value: &str) -> String {
    format!("{} {}", label(&format!("{}:", key)), value)
}
