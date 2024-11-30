use colored::*;
use std::path::Path;

pub const FILE_COLOR: Color = Color::White;
pub const DIRECTORY_COLOR: Color = Color::BrightBlue;
pub const SYMLINK_COLOR: Color = Color::BrightCyan;
pub const EXECUTABLE_COLOR: Color = Color::BrightGreen;
pub const SIZE_COLOR: Color = Color::Green;
pub const DATE_COLOR: Color = Color::BrightBlue;
pub const USER_COLOR: Color = Color::Cyan;
pub const GROUP_COLOR: Color = Color::BrightBlack;

pub const PERMISSION_DIR_COLOR: Color = Color::BrightBlue;
pub const PERMISSION_READ_COLOR: Color = Color::BrightCyan;
pub const PERMISSION_WRITE_COLOR: Color = Color::BrightYellow;
pub const PERMISSION_EXEC_COLOR: Color = Color::BrightRed;
pub const PERMISSION_NONE_COLOR: Color = Color::BrightBlack;

pub const RUST_COLOR: Color = Color::Red;
pub const SCRIPT_COLOR: Color = Color::Yellow;
pub const TYPESCRIPT_COLOR: Color = Color::Blue;
pub const MARKUP_COLOR: Color = Color::Magenta;
pub const STYLE_COLOR: Color = Color::Cyan;
pub const DOC_COLOR: Color = Color::White;
pub const MEDIA_COLOR: Color = Color::Magenta;
pub const ARCHIVE_COLOR: Color = Color::Red;
pub const DATA_COLOR: Color = Color::Yellow;

fn get_extension_color(path: &Path) -> Option<Color> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" | "toml" | "lock" => RUST_COLOR,
            "js" | "mjs" | "cjs" | "jsx" | "json" | "json5" | "yaml" | "yml" => SCRIPT_COLOR,
            "ts" | "tsx" | "d.ts" => TYPESCRIPT_COLOR,
            "html" | "htm" | "xml" | "vue" | "ejs" => MARKUP_COLOR,
            "css" | "scss" | "sass" | "less" | "conf" | "config" | "ini" | "env" => STYLE_COLOR,
            "py" | "pyi" | "pyw" | "sh" | "bash" | "zsh" | "fish" | "bat" | "cmd" | "ps1" => EXECUTABLE_COLOR,
            "md" | "rst" | "txt" | "doc" | "docx" | "pdf" | "org" | "wiki" => DOC_COLOR,
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "bmp" | "ico" | "webp" | "tiff" => MEDIA_COLOR,
            "zip" | "tar" | "gz" | "rar" | "7z" | "iso" | "dmg" | "exe" | "dll" => ARCHIVE_COLOR,
            "csv" | "tsv" | "sqlite" | "db" => DATA_COLOR,
            _ => FILE_COLOR,
        })
}

pub fn colorize_file_name(path: &Path) -> ColoredString {
    let name = path.file_name().unwrap().to_str().unwrap();
    
    if path.is_dir() {
        format!("{}/", name).color(DIRECTORY_COLOR).bold()
    } else if path.is_symlink() {
        name.color(SYMLINK_COLOR).italic().underline()
    } else if is_executable(path) {
        name.color(EXECUTABLE_COLOR).bold()
    } else if let Some(color) = get_extension_color(path) {
        name.color(color)
    } else {
        name.color(FILE_COLOR)
    }
}

pub fn colorize_size(size: u64) -> ColoredString {
    if size < 1024 {
        format!("{}B", size).color(SIZE_COLOR)
    } else if size < 1024 * 1024 {
        format!("{:.1}K", size as f64 / 1024.0).color(SIZE_COLOR)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}M", size as f64 / (1024.0 * 1024.0)).color(SIZE_COLOR)
    } else {
        format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0)).color(SIZE_COLOR)
    }
}

pub fn colorize_group(group: &str) -> ColoredString {
    group.color(GROUP_COLOR)
}

pub fn colorize_user(user: &str) -> ColoredString {
    user.color(USER_COLOR)
}

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

pub fn colorize_permissions(permissions: &Permissions) -> String {
    let mode = permissions.mode();
    let file_type = if mode & 0o40000 != 0 {
        "d".color(PERMISSION_DIR_COLOR)
    } else {
        "-".color(PERMISSION_NONE_COLOR)
    };
    let user = triplet(mode, 6);
    let group = triplet(mode, 3);
    let other = triplet(mode, 0);
    format!("{}{}{}{}", file_type, user, group, other)
}

fn triplet(mode: u32, shift: u32) -> String {
    let r = if mode >> (shift + 2) & 1 != 0 {
        "r".color(PERMISSION_READ_COLOR).to_string()
    } else {
        "-".color(PERMISSION_NONE_COLOR).to_string()
    };
    let w = if mode >> (shift + 1) & 1 != 0 {
        "w".color(PERMISSION_WRITE_COLOR).to_string()
    } else {
        "-".color(PERMISSION_NONE_COLOR).to_string()
    };
    let x = if mode >> shift & 1 != 0 {
        "x".color(PERMISSION_EXEC_COLOR).to_string()
    } else {
        "-".color(PERMISSION_NONE_COLOR).to_string()
    };
    format!("{}{}{}", r, w, x)
}

pub fn colorize_date(date: &std::time::SystemTime) -> ColoredString {
    let datetime: chrono::DateTime<chrono::Local> = (*date).into();
    datetime.format("%b %d %H:%M").to_string().color(DATE_COLOR)
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
