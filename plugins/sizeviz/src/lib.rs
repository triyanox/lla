use colored::Colorize;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use prost::Message as _;
use std::cmp;

pub struct FileSizeVisualizerPlugin;

impl FileSizeVisualizerPlugin {
    pub fn new() -> Self {
        FileSizeVisualizerPlugin
    }

    fn encode_error(&self, error: &str) -> Vec<u8> {
        use prost::Message;
        let error_msg = lla_plugin_interface::proto::PluginMessage {
            message: Some(
                lla_plugin_interface::proto::plugin_message::Message::ErrorResponse(
                    error.to_string(),
                ),
            ),
        };
        let mut buf = bytes::BytesMut::with_capacity(error_msg.encoded_len());
        error_msg.encode(&mut buf).unwrap();
        buf.to_vec()
    }

    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        match size {
            0 => "0 B".to_string(),
            s if s < KB => format!("{} B", s),
            s if s < MB => format!("{:.1} KB", s as f64 / KB as f64),
            s if s < GB => format!("{:.1} MB", s as f64 / MB as f64),
            s if s < TB => format!("{:.2} GB", s as f64 / GB as f64),
            s => format!("{:.2} TB", s as f64 / TB as f64),
        }
    }

    fn size_to_bar(size: u64, max_size: u64, max_width: usize) -> String {
        let normalized_size = (size as f64 / max_size as f64 * max_width as f64).ceil() as usize;
        let width = cmp::min(normalized_size, max_width);

        let full_blocks = "█".repeat(width);
        let remainder = (normalized_size as f64 * 8.0) as usize % 8;
        let partial_block = match remainder {
            1 => "▏",
            2 => "▎",
            3 => "▍",
            4 => "▌",
            5 => "▋",
            6 => "▊",
            7 => "▉",
            _ => "",
        };

        let spaces = " ".repeat(max_width.saturating_sub(width));
        format!("{}{}{}", full_blocks, partial_block, spaces)
    }

    fn size_to_color(size: u64) -> colored::Color {
        const KB: u64 = 1024;
        const KB_1: u64 = KB + 1;
        const KB_10: u64 = KB * 10;
        const KB_10_1: u64 = KB_10 + 1;
        const MB: u64 = KB * 1024;
        const MB_1: u64 = MB + 1;
        const MB_10: u64 = MB * 10;
        const MB_10_1: u64 = MB_10 + 1;
        const MB_100: u64 = MB * 100;
        const MB_100_1: u64 = MB_100 + 1;
        const GB: u64 = MB * 1024;

        match size {
            0..=KB => colored::Color::Green,
            KB_1..=KB_10 => colored::Color::BrightGreen,
            KB_10_1..=MB => colored::Color::Cyan,
            MB_1..=MB_10 => colored::Color::Blue,
            MB_10_1..=MB_100 => colored::Color::Yellow,
            MB_100_1..=GB => colored::Color::Red,
            _ => colored::Color::Magenta,
        }
    }

    fn get_percentage(size: u64, total_size: u64) -> f64 {
        if total_size == 0 {
            0.0
        } else {
            (size as f64 / total_size as f64) * 100.0
        }
    }
}

impl Plugin for FileSizeVisualizerPlugin {
    fn handle_raw_request(&mut self, request: &[u8]) -> Vec<u8> {
        let proto_msg = match proto::PluginMessage::decode(request) {
            Ok(msg) => msg,
            Err(e) => {
                let error_msg = proto::PluginMessage {
                    message: Some(Message::ErrorResponse(format!(
                        "Failed to decode request: {}",
                        e
                    ))),
                };
                let mut buf = bytes::BytesMut::with_capacity(error_msg.encoded_len());
                error_msg.encode(&mut buf).unwrap();
                return buf.to_vec();
            }
        };

        let response_msg = match proto_msg.message {
            Some(Message::GetName(_)) => Message::NameResponse(env!("CARGO_PKG_NAME").to_string()),
            Some(Message::GetVersion(_)) => {
                Message::VersionResponse(env!("CARGO_PKG_VERSION").to_string())
            }
            Some(Message::GetDescription(_)) => {
                Message::DescriptionResponse(env!("CARGO_PKG_DESCRIPTION").to_string())
            }
            Some(Message::GetSupportedFormats(_)) => {
                Message::FormatsResponse(proto::SupportedFormatsResponse {
                    formats: vec!["default".to_string(), "long".to_string()],
                })
            }
            Some(Message::Decorate(entry)) => {
                let mut entry = match lla_plugin_interface::DecoratedEntry::try_from(entry.clone())
                {
                    Ok(e) => e,
                    Err(e) => {
                        return self.encode_error(&format!("Failed to convert entry: {}", e));
                    }
                };

                if entry.path.is_file() {
                    let size = entry.metadata.size;
                    entry
                        .custom_fields
                        .insert("size".to_string(), size.to_string());
                }
                Message::DecoratedResponse(entry.into())
            }
            Some(Message::FormatField(req)) => {
                let entry = match req.entry {
                    Some(e) => match lla_plugin_interface::DecoratedEntry::try_from(e) {
                        Ok(entry) => entry,
                        Err(e) => {
                            return self.encode_error(&format!("Failed to convert entry: {}", e));
                        }
                    },
                    None => return self.encode_error("Missing entry in format field request"),
                };

                let formatted = match req.format.as_str() {
                    "long" => entry
                        .custom_fields
                        .get("size")
                        .and_then(|size_str| size_str.parse::<u64>().ok())
                        .map(|size| {
                            let max_size = 1_073_741_824;
                            let bar = Self::size_to_bar(size, max_size, 20);
                            let color = Self::size_to_color(size);
                            let formatted_size = Self::format_size(size);
                            let percentage = Self::get_percentage(size, max_size);

                            format!(
                                "\n{}\n{}\n{}\n{}",
                                format!(
                                    "┌─ {} ─{}",
                                    "Size".bright_cyan(),
                                    "─".repeat(40).bright_black()
                                ),
                                format!(
                                    "│ {} {}",
                                    bar.color(color),
                                    formatted_size.bright_yellow()
                                ),
                                format!(
                                    "│ {}% of reference (1GB)",
                                    format!("{:.2}", percentage).bright_magenta()
                                ),
                                format!("└{}", "─".repeat(50).bright_black())
                            )
                        }),
                    "default" => entry
                        .custom_fields
                        .get("size")
                        .and_then(|size_str| size_str.parse::<u64>().ok())
                        .map(|size| {
                            let max_size = 1_073_741_824;
                            let bar = Self::size_to_bar(size, max_size, 10);
                            let color = Self::size_to_color(size);
                            format!(
                                "{} {}",
                                bar.color(color),
                                Self::format_size(size).bright_yellow()
                            )
                        }),
                    _ => None,
                };
                Message::FieldResponse(proto::FormattedFieldResponse { field: formatted })
            }
            Some(Message::Action(_)) => Message::ActionResponse(proto::ActionResponse {
                success: true,
                error: None,
            }),
            _ => Message::ErrorResponse("Invalid request type".to_string()),
        };

        let response = proto::PluginMessage {
            message: Some(response_msg),
        };
        let mut buf = bytes::BytesMut::with_capacity(response.encoded_len());
        response.encode(&mut buf).unwrap();
        buf.to_vec()
    }
}

impl Default for FileSizeVisualizerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(FileSizeVisualizerPlugin);
