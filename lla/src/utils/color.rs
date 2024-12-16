use crate::commands::args::Args;
use crate::theme::{color_value_to_color, get_file_color, is_no_color, ColorValue, Theme};
use colored::*;
use std::path::Path;
use std::sync::OnceLock;

static CURRENT_THEME: OnceLock<Theme> = OnceLock::new();

pub fn set_theme(theme: Theme) {
    let _ = CURRENT_THEME.set(theme);
}

pub fn get_theme() -> &'static Theme {
    CURRENT_THEME.get_or_init(Theme::default)
}

fn get_color(color_value: &ColorValue) -> Color {
    color_value_to_color(color_value)
}

pub fn colorize_file_name(path: &Path) -> ColoredString {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| path.to_str().unwrap_or(""));

    if is_no_color() {
        return if path.is_dir() {
            format!("{}/", name).normal()
        } else {
            name.normal()
        };
    }

    let theme = get_theme();

    if path.is_dir() {
        if let Some(color) = get_file_color(path) {
            format!("{}/", name).color(color).bold()
        } else {
            format!("{}", name)
                .color(get_color(&theme.colors.directory))
                .bold()
        }
    } else if path.is_symlink() {
        name.color(get_color(&theme.colors.symlink))
            .italic()
            .underline()
    } else if is_executable(path) {
        name.color(get_color(&theme.colors.executable)).bold()
    } else if let Some(color) = get_extension_color(path) {
        name.color(color)
    } else {
        name.color(get_color(&theme.colors.file))
    }
}

pub fn colorize_file_name_with_icon(path: &Path, content: String) -> ColoredString {
    let parts: Vec<&str> = content.split(' ').collect();
    if parts.len() != 2 {
        return if is_no_color() {
            content.normal()
        } else {
            content.color(get_color(&get_theme().colors.file))
        };
    }

    let icon = parts[0];
    let name = parts[1];

    if is_no_color() {
        return if path.is_dir() {
            format!("{} {}", icon, name).normal()
        } else {
            format!("{} {}", icon, name).normal()
        };
    }

    let theme = get_theme();

    if path.is_dir() {
        if let Some(color) = get_file_color(path) {
            format!("{} {}", icon, name).color(color).bold()
        } else {
            format!("{} {}", icon, name)
                .color(get_color(&theme.colors.directory))
                .bold()
        }
    } else if path.is_symlink() {
        format!("{} {}", icon, name)
            .color(get_color(&theme.colors.symlink))
            .italic()
            .underline()
    } else if is_executable(path) {
        format!("{} {}", icon, name)
            .color(get_color(&theme.colors.executable))
            .bold()
    } else if let Some(color) = get_extension_color(path) {
        format!("{} {}", icon, name).color(color)
    } else {
        format!("{} {}", icon, name).color(get_color(&theme.colors.file))
    }
}

pub fn colorize_size(size: u64) -> ColoredString {
    let formatted = if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1}K", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}M", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
    };

    if is_no_color() {
        formatted.normal()
    } else {
        let theme = get_theme();
        formatted.color(get_color(&theme.colors.size))
    }
}

pub fn colorize_group(group: &str) -> ColoredString {
    if is_no_color() {
        group.normal()
    } else {
        group.color(get_color(&get_theme().colors.group))
    }
}

pub fn colorize_user(user: &str) -> ColoredString {
    if is_no_color() {
        user.normal()
    } else {
        user.color(get_color(&get_theme().colors.user))
    }
}

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

pub fn colorize_permissions(permissions: &Permissions) -> String {
    let mode = permissions.mode();

    if is_no_color() {
        return format_permissions_no_color(mode);
    }

    let theme = get_theme();

    let file_type = if mode & 0o40000 != 0 {
        "d".color(get_color(&theme.colors.permission_dir))
    } else {
        "-".color(get_color(&theme.colors.permission_none))
    };
    let user = triplet(mode, 6);
    let group = triplet(mode, 3);
    let other = triplet(mode, 0);
    format!("{}{}{}{}", file_type, user, group, other)
}

fn format_permissions_no_color(mode: u32) -> String {
    let file_type = if mode & 0o40000u32 != 0u32 { "d" } else { "-" };
    let read = |shift| {
        if mode >> shift & 0o4u32 != 0u32 {
            "r"
        } else {
            "-"
        }
    };
    let write = |shift| {
        if mode >> shift & 0o2u32 != 0u32 {
            "w"
        } else {
            "-"
        }
    };
    let exec = |shift| {
        if mode >> shift & 0o1u32 != 0u32 {
            "x"
        } else {
            "-"
        }
    };

    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        file_type,
        read(6),
        write(6),
        exec(6),
        read(3),
        write(3),
        exec(3),
        read(0),
        write(0),
        exec(0)
    )
}

fn triplet(mode: u32, shift: u32) -> String {
    let theme = get_theme();
    let r = if mode >> (shift + 2) & 1u32 != 0 {
        "r".color(get_color(&theme.colors.permission_read))
            .to_string()
    } else {
        "-".color(get_color(&theme.colors.permission_none))
            .to_string()
    };
    let w = if mode >> (shift + 1) & 1u32 != 0 {
        "w".color(get_color(&theme.colors.permission_write))
            .to_string()
    } else {
        "-".color(get_color(&theme.colors.permission_none))
            .to_string()
    };
    let x = if mode >> shift & 1u32 != 0 {
        "x".color(get_color(&theme.colors.permission_exec))
            .to_string()
    } else {
        "-".color(get_color(&theme.colors.permission_none))
            .to_string()
    };
    format!("{}{}{}", r, w, x)
}

pub fn colorize_date(date: &std::time::SystemTime) -> ColoredString {
    let datetime: chrono::DateTime<chrono::Local> = (*date).into();
    let formatted = datetime.format("%b %d %H:%M").to_string();

    if is_no_color() {
        formatted.normal()
    } else {
        formatted.color(get_color(&get_theme().colors.date))
    }
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

fn get_extension_color(path: &Path) -> Option<Color> {
    if is_no_color() {
        return None;
    }
    use crate::theme::get_file_color;
    get_file_color(path)
}

pub struct ColorState {
    pub no_color: bool,
}

impl ColorState {
    pub fn new(args: &Args) -> Self {
        Self {
            no_color: args.no_color,
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.no_color
    }
}
