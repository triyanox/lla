use super::{TextBlock, TextStyle};
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp;
use std::time::Duration;

pub struct Spinner {
    progress_bar: ProgressBar,
}

impl Spinner {
    pub fn new() -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.enable_steady_tick(Duration::from_millis(80));
        Self { progress_bar: pb }
    }

    pub fn set_status(&self, status: impl Into<String>) {
        self.progress_bar.set_message(status.into());
    }

    pub fn finish(&self) {
        self.progress_bar.finish_and_clear();
    }

    pub fn finish_with_message(&self, msg: impl Into<String>) {
        self.progress_bar.finish_with_message(msg.into());
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.finish();
    }
}

pub struct HelpFormatter {
    title: String,
    sections: Vec<(String, Vec<(String, String, Vec<String>)>)>,
}

impl HelpFormatter {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }

    pub fn add_section(&mut self, title: impl Into<String>) -> &mut Self {
        self.sections.push((title.into(), Vec::new()));
        self
    }

    pub fn add_command(
        &mut self,
        command: impl Into<String>,
        description: impl Into<String>,
        examples: Vec<String>,
    ) -> &mut Self {
        if let Some((_, commands)) = self.sections.last_mut() {
            commands.push((command.into(), description.into(), examples));
        }
        self
    }

    pub fn render(&self, colors: &std::collections::HashMap<String, String>) -> String {
        let mut output = String::new();
        let default_color = "white".to_string();
        let title_color = colors.get("success").unwrap_or(&default_color);
        let section_color = colors.get("info").unwrap_or(&default_color);
        let cmd_color = colors.get("name").unwrap_or(&default_color);

        output.push_str(
            &TextBlock::new(&self.title)
                .color(title_color)
                .style(TextStyle::Bold)
                .build(),
        );
        output.push_str("\n\n");

        for (section_title, commands) in &self.sections {
            output.push_str(&TextBlock::new(section_title).color(section_color).build());
            output.push_str("\n\n");

            for (command, description, examples) in commands {
                output.push_str("  ");
                output.push_str(
                    &TextBlock::new(command)
                        .color(cmd_color)
                        .style(TextStyle::Bold)
                        .build(),
                );
                output.push_str("\n    ");
                output.push_str(description);
                output.push_str("\n");

                if !examples.is_empty() {
                    output.push_str("\n    Examples:\n");
                    for example in examples {
                        output.push_str("      • ");
                        output.push_str(example);
                        output.push_str("\n");
                    }
                }
                output.push_str("\n");
            }
        }

        output
    }
}

pub struct KeyValue {
    key: String,
    value: String,
    key_color: Option<String>,
    value_color: Option<String>,
    key_width: Option<usize>,
}

impl KeyValue {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            key_color: None,
            value_color: None,
            key_width: None,
        }
    }

    pub fn key_color(mut self, color: impl Into<String>) -> Self {
        self.key_color = Some(color.into());
        self
    }

    pub fn value_color(mut self, color: impl Into<String>) -> Self {
        self.value_color = Some(color.into());
        self
    }

    pub fn key_width(mut self, width: usize) -> Self {
        self.key_width = Some(width);
        self
    }

    pub fn render(&self) -> String {
        let key = if let Some(color) = &self.key_color {
            TextBlock::new(&self.key).color(color).build()
        } else {
            self.key.clone()
        };

        let value = if let Some(color) = &self.value_color {
            TextBlock::new(&self.value).color(color).build()
        } else {
            self.value.clone()
        };

        if let Some(width) = self.key_width {
            format!("{:width$} {}", key, value, width = width)
        } else {
            format!("{} {}", key, value)
        }
    }
}

pub struct List {
    items: Vec<String>,
}

impl List {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: impl Into<String>) -> &mut Self {
        self.items.push(item.into());
        self
    }

    pub fn style(self, _style: BoxStyle) -> Self {
        self
    }

    pub fn key_width(self, _width: usize) -> Self {
        self
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push('┌');
        output.push('─');
        output.push('\n');

        for item in &self.items {
            output.push('│');
            output.push(' ');
            output.push_str(item);
            output.push('\n');
        }

        output.push('└');
        output.push('─');
        output.push('\n');
        output
    }
}

#[derive(Clone, Copy)]
pub enum BoxStyle {
    Minimal,
    Rounded,
    Double,
    Heavy,
    Dashed,
}

impl BoxStyle {
    fn get_chars(&self) -> BoxChars {
        match self {
            BoxStyle::Minimal => BoxChars {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '─',
                vertical: '│',
                left_t: '├',
                right_t: '┤',
                top_t: '┬',
                bottom_t: '┴',
                cross: '┼',
            },
            BoxStyle::Rounded => BoxChars {
                top_left: '╭',
                top_right: '╮',
                bottom_left: '╰',
                bottom_right: '╯',
                horizontal: '─',
                vertical: '│',
                left_t: '├',
                right_t: '┤',
                top_t: '┬',
                bottom_t: '┴',
                cross: '┼',
            },
            BoxStyle::Double => BoxChars {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
                left_t: '╠',
                right_t: '╣',
                top_t: '╦',
                bottom_t: '╩',
                cross: '╬',
            },
            BoxStyle::Heavy => BoxChars {
                top_left: '┏',
                top_right: '┓',
                bottom_left: '┗',
                bottom_right: '┛',
                horizontal: '━',
                vertical: '┃',
                left_t: '┣',
                right_t: '┫',
                top_t: '┳',
                bottom_t: '┻',
                cross: '╋',
            },
            BoxStyle::Dashed => BoxChars {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '┄',
                vertical: '┆',
                left_t: '├',
                right_t: '┤',
                top_t: '┬',
                bottom_t: '┴',
                cross: '┼',
            },
        }
    }
}

#[allow(dead_code)]
struct BoxChars {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
    left_t: char,
    right_t: char,
    top_t: char,
    bottom_t: char,
    cross: char,
}

pub struct BoxComponent {
    content: String,
    style: BoxStyle,
    width: Option<usize>,
    padding: usize,
    title: Option<String>,
}

impl BoxComponent {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: BoxStyle::Minimal,
            width: None,
            padding: 0,
            title: None,
        }
    }

    pub fn style(mut self, style: BoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn render(&self) -> String {
        let chars = self.style.get_chars();
        let mut output = String::new();
        output.push('\n');
        let lines: Vec<&str> = self.content.lines().collect();

        let content_width = lines
            .iter()
            .map(|line| console::measure_text_width(line))
            .max()
            .unwrap_or(0);
        let title_width = self
            .title
            .as_ref()
            .map(|t| console::measure_text_width(t))
            .unwrap_or(0);
        let inner_width = cmp::max(content_width, title_width) + self.padding * 2;
        let total_width = self.width.unwrap_or(inner_width);

        output.push(chars.top_left);
        if let Some(title) = &self.title {
            output.push(chars.horizontal);
            output.push(' ');
            output.push_str(title);
            output.push(' ');
            let remaining = total_width.saturating_sub(title_width + 4);
            output.push_str(&chars.horizontal.to_string().repeat(remaining));
        } else {
            output.push_str(&chars.horizontal.to_string().repeat(total_width));
        }
        output.push(chars.top_right);
        output.push('\n');

        for _ in 0..self.padding {
            output.push(chars.vertical);
            output.push_str(&" ".repeat(total_width));
            output.push(chars.vertical);
            output.push('\n');
        }

        for line in lines {
            output.push(chars.vertical);
            output.push_str(&" ".repeat(self.padding));
            output.push_str(line);
            let padding =
                total_width.saturating_sub(console::measure_text_width(line) + self.padding);
            output.push_str(&" ".repeat(padding));
            output.push(chars.vertical);
            output.push('\n');
        }

        for _ in 0..self.padding {
            output.push(chars.vertical);
            output.push_str(&" ".repeat(total_width));
            output.push(chars.vertical);
            output.push('\n');
        }

        output.push(chars.bottom_left);
        output.push_str(&chars.horizontal.to_string().repeat(total_width));
        output.push(chars.bottom_right);
        output.push('\n');

        output
    }
}
