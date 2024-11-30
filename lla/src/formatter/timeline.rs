use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use chrono::{DateTime, Duration, Local};
use colored::*;
use lla_plugin_interface::DecoratedEntry;
use std::collections::BTreeMap;

pub struct TimelineFormatter;

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
        plugin_manager: &PluginManager,
        _depth: Option<usize>,
    ) -> Result<String> {
        if files.is_empty() {
            return Ok(String::new());
        }

        let mut groups: BTreeMap<TimeGroup, Vec<&DecoratedEntry>> = BTreeMap::new();

        for file in files {
            let modified = file.metadata.modified()?;
            let dt: DateTime<Local> = modified.into();
            let group = TimeGroup::from_datetime(dt);
            groups.entry(group).or_default().push(file);
        }

        let mut output = String::new();
        let time_format = "%H:%M:%S";
        let date_format = "%Y-%m-%d";

        for (group, entries) in groups.iter() {
            let header = match group {
                TimeGroup::Today => group.display_name().to_string(),
                TimeGroup::Yesterday => {
                    let yesterday = Local::now().date_naive() - Duration::days(1);
                    format!(
                        "{} ({})",
                        group.display_name(),
                        yesterday.format(date_format)
                    )
                }
                _ => group.display_name().to_string(),
            };
            output.push_str(&format!("\n{}\n", header.bold().blue()));
            output.push_str(&"─".repeat(header.len()));
            output.push('\n');

            let mut entries = entries.to_vec();
            entries.sort_by_key(|e| std::cmp::Reverse(e.metadata.modified().unwrap()));

            for entry in entries {
                let name = colorize_file_name(&entry.path);
                let modified = entry.metadata.modified()?;
                let dt: DateTime<Local> = modified.into();

                let datetime_str = match group {
                    TimeGroup::Today | TimeGroup::Yesterday => dt.format(time_format).to_string(),
                    _ => {
                        format!("{} {}", dt.format(date_format), dt.format(time_format))
                    }
                };

                let plugin_fields = plugin_manager.format_fields(entry, "timeline").join(" ");
                let git_info = if let Some(git_field) = plugin_fields
                    .split_whitespace()
                    .find(|s| s.contains("commit:"))
                {
                    format!(" [{}]", git_field.bright_yellow())
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{} {} {}{}",
                    datetime_str.bright_black(),
                    name,
                    git_info,
                    if plugin_fields.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", plugin_fields)
                    }
                ));
                output.push('\n');
            }
        }

        Ok(output)
    }
}
