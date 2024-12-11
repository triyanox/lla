# `lla` - A Modern Alternative to ls

<a href="https://repology.org/project/lla-ls/versions">
  <img src="https://repology.org/badge/version-for-repo/aur/lla-ls.svg" alt="AUR package">
</a>
<a href="https://repology.org/project/lla-ls/versions">
  <img src="https://repology.org/badge/version-for-repo/crates_io/lla-ls.svg" alt="crates.io package">
</a>
<a href="https://repology.org/project/lla-ls/versions">
  <img src="https://repology.org/badge/version-for-repo/homebrew/lla-ls.svg" alt="Homebrew package">
</a>
<a href="https://repology.org/project/lla-ls/versions">
  <img src="https://repology.org/badge/version-for-repo/nix_unstable/lla-ls.svg" alt="nixpkgs unstable package">
</a>
<a href="https://repology.org/project/lla-ls/versions">
  <img src="https://repology.org/badge/version-for-repo/pkgsrc_current/lla-ls.svg" alt="pkgsrc current package">
</a>

`lla` is a high-performance file explorer written in Rust that enhances the traditional `ls` command with modern features, rich formatting options, and a powerful plugin system.

## Display Formats

### Default View

Quick and clean directory listing

```bash
lla
```

![default](https://github.com/user-attachments/assets/8e170a91-0084-406c-a5d2-746cce8fc33d)

### Long Format

Detailed file information with metadata

```bash
lla -l
```

![long](https://github.com/user-attachments/assets/68682c3e-5190-4708-9472-df850a66e6fc)

### Tree View

Hierarchical directory visualization

```bash
lla -t
```

![tree](https://github.com/user-attachments/assets/f0a38062-bd10-4891-b621-13205a3b7268)

### Table View

Structured data display

```bash
lla -T
```

![table](https://github.com/user-attachments/assets/49609647-970e-4d3b-bb87-32693989ee13)

### Grid View

Organized layout for better readability

```bash
lla -g
```

![grid](https://github.com/user-attachments/assets/29e4cc3e-81e6-4b08-80fc-8272f2ac2433)

### Git-Aware View

Repository status and insights

```bash
lla -G
```

![git](https://github.com/user-attachments/assets/a0d68240-adfe-411b-90e3-8c6749c063b9)

### Timeline View

Group files by dates

```bash
lla --timeline
```

![timeline](https://github.com/user-attachments/assets/11cc4b85-8a53-457f-a0e6-38e3c9f83b83)

## Core Features

- **Smart Display Formats**

  - Default view for quick browsing
  - Long format (`-l`) with detailed metadata
  - Tree view (`-t`) for directory visualization
  - Grid view (`-g`) for organized layouts
  - Table view (`-T`) for structured data
  - Size map (`-S`) for visual space analysis [experimental]
  - Timeline view (`--timeline`) for temporal organization
  - Git-aware view (`-G`) for repository insights

- **Intelligent Organization**

  - Sort by name, size, or date (`-s`)
  - Filter by name or extension (`-f`)
  - Recursive listing with depth control (`-d`)
  - Performance-optimized for large directories

## Filter System

`lla` provides a powerful and flexible filtering system to help you find files efficiently:

### Basic Filtering

```bash
# Simple pattern matching
lla -f "test"              # Find files containing "test"
lla -f "test" -c          # Case-sensitive search for "test"
lla -f ".rs"              # Find files with .rs extension
```

### Advanced Filter Types

1. **Pattern Filter**

```bash
# Multiple patterns (OR operation)
lla -f "test,spec"         # Match files containing either "test" or "spec"

# AND operation with + prefix
lla -f "+test,api"         # Match files containing both "test" AND "api"
```

2. **Regular Expression Filter**

```bash
# Use regex: prefix for regular expressions
lla -f "regex:^test.*\.rs$"   # Match Rust files starting with "test"
lla -f "regex:\d{4}"          # Match files containing 4 digits
```

3. **Glob Pattern Filter**

```bash
# Use glob: prefix for glob patterns
lla -f "glob:*.{rs,toml}"     # Match .rs or .toml files
lla -f "glob:test_*"          # Match files starting with test_
```

### Composite Filters

Combine multiple filters using logical operators:

```bash
# AND operation
lla -f "test AND .rs"         # Files containing "test" AND having .rs extension
lla -f "regex:^test AND glob:*.rs"

# OR operation
lla -f "test OR spec"         # Files containing either "test" OR "spec"
lla -f ".rs OR .toml"         # Files with either extension

# NOT operation
lla -f "NOT test"             # Files NOT containing "test"
lla -f "glob:*.rs AND NOT test"

# XOR operation (exclusive OR)
lla -f "test XOR spec"        # Files containing either "test" or "spec", but not both
```

### Filter with Other Features

Combine filters with other `lla` features:

```bash
# Filter with tree view
lla -f "test" -t            # Show tree view of files containing "test"

# Filter with depth control
lla -f ".rs" -t -d 3        # Show Rust files in tree view, max depth 3

# Filter and sort
lla -f "test" -ls size      # Show matching files sorted by size

# Filter in long format
lla -f "test" -l            # Detailed view of matching files
```

### Tips for Filtering

- Use quotes around filter patterns to prevent shell interpretation
- Case-sensitive filtering is available with the `-c` flag
- Filters work on both filenames and paths
- Multiple patterns in comma-separated lists are treated as OR conditions
- Use `+` prefix for AND conditions in comma-separated lists
- Combine with `--tree` to see the full path context of matches

- **Plugin Ecosystem**
  - Git integration for repository insights
  - File categorization and tagging
  - Code complexity analysis
  - Content keyword search
  - File hash calculation
  - Size visualization
  - Duplicate detection
  - Extended metadata display

## Quick Start

1. Install `lla`:

```bash
# Using Cargo
cargo install lla

# On macOS
brew install lla

# On Arch Linux (btw)
paru -S lla

# On NetBSD (we see you)
pkgin install lla
```

2. Initialize your setup:

`lla` uses a TOML configuration file located at `~/.config/lla/config.toml`.

```bash
# Create default config
lla init

# View your config
lla config
```

3. Start exploring:

```bash
# Basic usage
lla                     # List current directory
lla -l                  # Long format with details
lla -t                  # Tree view
lla -g                  # Grid view
lla -T                  # Table view
lla -S                  # Size map view
lla --timeline          # Timeline view
lla -G                  # Git-aware view

# Advanced usage
lla -ls size           # Sort by size in long format
lla -f .rs            # Show only Rust files
# Case-insensitive search for files containing "test"
lla -f test
# Case-sensitive search for files containing "Test"
lla -f test -c
# Regular expression search
lla -f "regex:.*\.rs$"
# Glob pattern search
lla -f "glob:*.{rs,toml}"
# Composite filters
lla -f "regex:.*\.rs$ AND NOT test"
lla -f ".rs OR .toml"
lla -f "test AND glob:*.rs"

# Tree view with max depth 3
lla -t -d 3
```

## Plugin System

https://github.com/user-attachments/assets/6064b70e-1275-4d60-88ed-3370c0068ebd

`lla` uses a plugin system to extend its functionality so you can enhance it with custom functionality based on your needs.
You can install plugins from a local directory or from a Git repository.
You can find official plugins [here](https://github.com/triyanox/lla/blob/main/plugins.md).

1. Install plugins:

```bash
# From Git repository
lla install --git https://github.com/user/plugin

# From local directory
lla install --dir path/to/plugin
```

2. Manage plugins:

```bash
# Interactive plugin manager
lla use

# Enable plugins
lla --enable-plugin git_status
lla --enable-plugin keyword_search

# Disable plugins
lla --disable-plugin git_status

# Update plugins
lla update              # Update all plugins
lla update plugin_name  # Update specific plugin
```

3. Plugin actions:

```bash
# Execute plugin actions
lla plugin --name keyword_search --action set-keywords --args "TODO" "FIXME"

# Example: Code snippet management through `code_snippet_extractor` plugin
lla plugin --name code_snippet_extractor --action extract --args "path/to/file.rs" "snippet_name" 10 20  # Extract lines 10-20
lla plugin --name code_snippet_extractor --action list --args "path/to/file.rs"                        # List snippets in file
lla plugin --name code_snippet_extractor --action get --args "path/to/file.rs" "snippet_name"          # Get snippet content
lla plugin --name code_snippet_extractor --action search --args "query"                                # Search snippets
lla plugin --name code_snippet_extractor --action add-tags --args "path/to/file.rs" "snippet" "tag1"   # Add tags
lla plugin --name code_snippet_extractor --action remove-tags --args "path/to/file.rs" "snippet" "tag1" # Remove tags
lla plugin --name code_snippet_extractor --action export --args "path/to/file.rs"                      # Export snippets to JSON
lla plugin --name code_snippet_extractor --action import --args "path/to/file.rs" "json_data"          # Import snippets
lla plugin --name code_snippet_extractor --action help                                                 # Show help
```

## Configuration

Your config lives at `~/.config/lla/config.toml`:

```toml
# Core settings
default_sort = "name"          # name, size, date
default_format = "default"     # default, long, tree, grid
enabled_plugins = ["git_status", "file_hash"]
plugins_dir = "/home/user/.config/lla/plugins"
default_depth = 3

# Performance settings
[formatters.tree]
max_lines = 20000             # Max entries in tree view
[listers.recursive]
max_entries = 20000           # Max entries in recursive listing
```

Modify settings via CLI:

```bash
lla config --set default_sort size
lla config --set default_format long
lla config --set plugins_dir /custom/path
lla config --set default_depth 5
```

## Plugin Development

Develop custom plugins to extend `lla`'s functionality. Plugins are dynamic libraries that implement the `Plugin` trait from the [lla_plugin_interface](https://github.com/triyanox/lla/tree/main/lla_plugin_interface) crate. The plugin system uses a message-passing architecture to ensure ABI compatibility between different Rust versions.

1. Create a new plugin:

```bash
cargo new --lib my_lla_plugin
```

2. Configure `Cargo.toml`:

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
lla_plugin_interface = "*"
serde = { version = "1.0", features = ["derive"] }

[lib]
crate-type = ["cdylib"]
```

3. Implement the plugin interface:

```rust
use lla_plugin_interface::{Plugin, PluginRequest, PluginResponse, DecoratedEntry, CliArg};
use std::collections::HashMap;

pub struct MyPlugin {
    // Your plugin state here
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            // Initialize your plugin
        }
    }
}

impl Plugin for MyPlugin {
    fn handle_request(&mut self, request: PluginRequest) -> PluginResponse {
        match request {
            PluginRequest::GetName => {
                PluginResponse::Name("my_plugin".to_string())
            }
            PluginRequest::GetVersion => {
                PluginResponse::Version(env!("CARGO_PKG_VERSION").to_string())
            }
            PluginRequest::GetDescription => {
                PluginResponse::Description(env!("CARGO_PKG_DESCRIPTION").to_string())
            }
            PluginRequest::GetSupportedFormats => {
                PluginResponse::SupportedFormats(vec!["default".to_string(), "long".to_string()])
            }
            PluginRequest::Decorate(mut entry) => {
                // Add custom fields to the entry
                entry.custom_fields.insert("my_field".to_string(), "value".to_string());
                PluginResponse::Decorated(entry)
            }
            PluginRequest::FormatField(entry, format) => {
                // Format a specific field for display
                PluginResponse::FormattedField(Some("formatted value".to_string()))
            }
            PluginRequest::PerformAction(action, args) => {
                match action.as_str() {
                    "my-action" => {
                        // Perform custom action
                        PluginResponse::ActionResult(Ok(()))
                    }
                    _ => PluginResponse::Error(format!("Unknown action: {}", action))
                }
            }
        }
    }
}

lla_plugin_interface::declare_plugin!(MyPlugin);
```

### Plugin Architecture

The new plugin system uses a message-passing architecture to ensure ABI compatibility between different Rust versions. Instead of direct trait method calls, plugins communicate through serializable request/response messages:

- `PluginRequest`: Represents requests from the main application to the plugin

  - `GetName`: Get the plugin name
  - `GetVersion`: Get the plugin version
  - `GetDescription`: Get the plugin description
  - `GetSupportedFormats`: Get supported display formats
  - `Decorate`: Add custom fields to an entry
  - `FormatField`: Format a specific field for display
  - `PerformAction`: Execute a custom action

- `PluginResponse`: Represents responses from the plugin back to the application
  - `Name`: Plugin name
  - `Version`: Plugin version
  - `Description`: Plugin description
  - `SupportedFormats`: List of supported formats
  - `Decorated`: Modified entry with custom fields
  - `FormattedField`: Formatted field value
  - `ActionResult`: Result of a custom action
  - `Error`: Error message

The `DecoratedEntry` struct now uses a platform-independent `EntryMetadata` struct instead of the OS-specific `std::fs::Metadata`, ensuring consistent behavior across different platforms and Rust versions.

4. Build your plugin:

```bash
cargo build --release
```

5. Install your plugin:

```bash
lla install --dir path/to/plugin
```

or

```bash
lla install --git https://github.com/user/plugin
```

### Plugin Interface

The [lla_plugin_interface](https://github.com/triyanox/lla/tree/main/lla_plugin_interface) crate provides the following key components:

- `Plugin` trait: Core interface for handling plugin requests and responses
- `PluginRequest`/`PluginResponse` enums: Message types for plugin communication
- `DecoratedEntry` struct: Represents a file entry with metadata and custom fields
- `EntryMetadata` struct: Platform-independent file metadata

## Contributing

Contributions are welcome! Please feel free to submit pull requests, report bugs, and suggest features.

1. Fork the repository
2. Create your feature branch (git checkout -b feature/new-feature)
3. Commit your changes (git commit -m 'Add some new-feature')
4. Push to the branch (git push origin feature/new-feature)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
