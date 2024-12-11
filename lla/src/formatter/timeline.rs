use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::{colorize_file_name, colorize_file_name_with_icon};
use crate::utils::icons::format_with_icon;
use chrono::{DateTime, Duration, Local};
use colored::*;
use lla_plugin_interface::proto::DecoratedEntry;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub struct TimelineFormatter {
    pub show_icons: bool,
}

impl TimelineFormatter {
    pub fn new(show_icons: bool) -> Self {
        Self { show_icons }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum TimeGroup {
    Today,
    Yesterday,
    LastWeek,
    LastMonth,
    Older,
}

impl TimeGroup {
    fn from_datetime(dt: DateTime<Local>) -> Self {
        let now = Local::now();
        let today = now.date_naive();
        let yesterday = today - Duration::days(1);
        let last_week = today - Duration::days(7);
        let last_month = today - Duration::days(30);

        let file_date = dt.date_naive();

        if file_date == today {
            TimeGroup::Today
        } else if file_date == yesterday {
            TimeGroup::Yesterday
        } else if file_date > last_week {
            TimeGroup::LastWeek
        } else if file_date > last_month {
            TimeGroup::LastMonth
        } else {
            TimeGroup::Older
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            TimeGroup::Today => "Today",
            TimeGroup::Yesterday => "Yesterday",
            TimeGroup::LastWeek => "Last Week",
            TimeGroup::LastMonth => "Last Month",
            TimeGroup::Older => "Older",
        }
    }
}

impl FileFormatter for TimelineFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &mut PluginManager,
        _depth: Option<usize>,
    ) -> Result<String> {
        if files.is_empty() {
            return Ok(String::new());
        }

        let mut groups: BTreeMap<TimeGroup, Vec<&DecoratedEntry>> = BTreeMap::new();

        for file in files {
            let modified = file.metadata.as_ref().map_or(0, |m| m.modified);
            let modified = UNIX_EPOCH + std::time::Duration::from_secs(modified);
            let dt = DateTime::<Local>::from(modified);
            let group = TimeGroup::from_datetime(dt);
            groups.entry(group).or_default().push(file);
        }

        let mut output = String::new();
        let time_format = "%H:%M:%S";
        let date_format = "%Y-%m-%d";

        for (group, entries) in groups {
            output.push_str(&format!(
                "{}\n{}\n\n",
                group.display_name().bright_blue().bold(),
                "─".repeat(40).bright_black()
            ));

            for entry in entries {
                let modified = entry.metadata.as_ref().map_or(0, |m| m.modified);
                let modified = UNIX_EPOCH + std::time::Duration::from_secs(modified);
                let dt = DateTime::<Local>::from(modified);

                let time_str = dt.format(time_format).to_string().bright_black();
                let date_str = if group == TimeGroup::Older {
                    dt.format(date_format).to_string().bright_black()
                } else {
                    "".bright_black()
                };

                let path = Path::new(&entry.path);
                let colored_name = colorize_file_name(path).to_string();
                let name = colorize_file_name_with_icon(
                    path,
                    format_with_icon(path, colored_name, self.show_icons),
                )
                .to_string();

                let plugin_fields = plugin_manager.format_fields(entry, "timeline").join(" ");
                let git_info = if let Some(git_field) = plugin_fields
                    .split_whitespace()
                    .find(|s| s.contains("commit:"))
                {
                    format!(" {}", git_field.bright_yellow())
                } else {
                    String::new()
                };

                output.push_str(&format!("{} {} {}{}\n", time_str, date_str, name, git_info));
            }
            output.push('\n');
        }

        Ok(output)
    }
}
