use colored::*;
use std::path::Path;

pub fn colorize_file_name(path: &Path) -> ColoredString {
    let name = path.file_name().unwrap().to_str().unwrap();
    if path.is_dir() {
        format!("{}/", name).bright_blue().bold()
    } else if path.is_symlink() {
        name.bright_cyan().italic()
    } else if is_executable(path) {
        name.bright_green()
    } else {
        name.normal()
    }
}

pub fn colorize_size(size: u64) -> ColoredString {
    if size < 1024 {
        format!("{}B", size).green()
    } else if size < 1024 * 1024 {
        format!("{:.1}K", size as f64 / 1024.0).green()
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}M", size as f64 / (1024.0 * 1024.0)).green()
    } else {
        format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0)).green()
    }
}

pub fn colorize_group(group: &str) -> ColoredString {
    group.bright_black()
}

pub fn colorize_user(user: &str) -> ColoredString {
    user.cyan()
}

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

pub fn colorize_permissions(permissions: &Permissions) -> String {
    let mode = permissions.mode();
    let file_type = if mode & 0o40000 != 0 {
        "d".bright_blue()
    } else {
        "-".bright_black()
    };
    let user = triplet(mode, 6);
    let group = triplet(mode, 3);
    let other = triplet(mode, 0);
    format!("{}{}{}{}", file_type, user, group, other)
}

fn triplet(mode: u32, shift: u32) -> String {
    let r = if mode >> (shift + 2) & 1 != 0 {
        "r".bright_cyan().to_string()
    } else {
        "-".bright_black().to_string()
    };
    let w = if mode >> (shift + 1) & 1 != 0 {
        "w".bright_yellow().to_string()
    } else {
        "-".bright_black().to_string()
    };
    let x = if mode >> shift & 1 != 0 {
        "x".bright_red().to_string()
    } else {
        "-".bright_black().to_string()
    };
    format!("{}{}{}", r, w, x)
}
pub fn colorize_date(date: &std::time::SystemTime) -> ColoredString {
    let datetime: chrono::DateTime<chrono::Local> = (*date).into();
    datetime.format("%b %d %H:%M").to_string().bright_blue()
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        if let Ok(metadata) = path.metadata() {
            return metadata.permissions().mode() & 0o111 != 0;
        }
    }
    false
}
