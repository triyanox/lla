use super::{TextBlock, TextStyle};
use indicatif::{ProgressBar, ProgressStyle};
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

pub enum BoxStyle {
    Minimal,
    Rounded,
    Double,
    Heavy,
    Dashed,
}

pub struct BoxComponent {
    content: String,
}

impl BoxComponent {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    pub fn style(self, _style: BoxStyle) -> Self {
        self
    }

    pub fn width(self, _width: usize) -> Self {
        self
    }

    pub fn padding(self, _padding: usize) -> Self {
        self
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push('┌');
        output.push('─');
        output.push('\n');

        for line in self.content.lines() {
            output.push('│');
            output.push(' ');
            output.push_str(line);
            output.push('\n');
        }

        output.push('└');
        output.push('─');
        output.push('\n');
        output
    }
}
