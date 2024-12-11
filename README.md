# `lla` - Blazing Fast `ls` Replacement with Superpowers

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

## Overview

`lla` is a high-performance file explorer written in Rust that enhances the traditional `ls` command with modern features, rich formatting options, and a powerful plugin system.

## Table of Contents

- [Installation](#installation)
- [Features](#features)
  - [Display Formats](#display-formats)
  - [Core Features](#core-features)
- [Usage](#usage)
  - [Command Reference](#command-reference)
  - [Filter System](#filter-system)
  - [Plugin System](#plugin-system)
- [Configuration](#configuration)
- [Development](#development)
  - [Plugin Development](#plugin-development)
  - [Contributing](#contributing)
- [License](#license)

## Installation

```bash
# Using Cargo
cargo install lla

# On macOS
brew install lla

# On Arch Linux
paru -S lla

# On NetBSD
pkgin install lla
```

After installation, initialize your setup:

```bash
# Create default config
lla init

# View your config
lla config
```

## Features

### Display Formats

#### Default View

Quick and clean directory listing

```bash
lla
```

![default](https://github.com/user-attachments/assets/8e170a91-0084-406c-a5d2-746cce8fc33d)

#### Long Format

Detailed file information with metadata

```bash
lla -l
```

![long](https://github.com/user-attachments/assets/68682c3e-5190-4708-9472-df850a66e6fc)

#### Tree View

Hierarchical directory visualization

```bash
lla -t
```

![tree](https://github.com/user-attachments/assets/f0a38062-bd10-4891-b621-13205a3b7268)

#### Table View

Structured data display

```bash
lla -T
```

![table](https://github.com/user-attachments/assets/49609647-970e-4d3b-bb87-32693989ee13)

#### Grid View

Organized layout for better readability

```bash
lla -g
```

![grid](https://github.com/user-attachments/assets/29e4cc3e-81e6-4b08-80fc-8272f2ac2433)

#### Git-Aware View

Repository status and insights

```bash
lla -G
```

![git](https://github.com/user-attachments/assets/19ae3be6-4ddc-4415-ac09-9b0e72ece662)

#### Timeline View

Group files by dates

```bash
lla --timeline
```

![timeline](https://github.com/user-attachments/assets/2bb9f80c-3059-4362-9f47-378fb0a1db18)

#### Sizemap View

Visualize file sizes relative to each other

```bash
lla -S
```

![sizemap](https://github.com/user-attachments/assets/9efc39a6-f21d-4b78-bad1-eb957beae6e9)


#### Icons support

Icons are supported in all formats simply by using the `--icons` flag.

```bash
lla --icons
```

![icons](https://github.com/user-attachments/assets/be4f95c3-014a-427b-98a9-ef8e2aa53877)

### Core Features

**Display Options**

- Long format (`-l`, `--long`): Detailed file information with metadata
- Tree view (`-t`, `--tree`): Hierarchical directory visualization
- Table view (`-T`, `--table`): Structured data display
- Grid view (`-g`, `--grid`): Organized layout for better readability
- Size map (`-S`, `--sizemap`): Visual representation of file sizes
- Timeline view (`--timeline`): Group files by time periods
- Git-aware view (`-G`, `--git`): Repository status and information
- Icons (`--icons`): Show icons for files and directories

**Organization & Sorting**

- Sort files (`-s`, `--sort`):
  - By name (default)
  - By size
  - By date (modification time)
- Sort modifiers:
  - Reverse order (`-r`, `--sort-reverse`)
  - Directories first (`--sort-dirs-first`)
  - Case-sensitive (`--sort-case-sensitive`)
  - Natural number sorting (`--sort-natural`)
- Recursive listing with depth control (`-d`, `--depth`)
- Performance optimization for large directories:
  - Configurable maximum entries for tree view (default: 20,000)
  - Configurable maximum entries for recursive listing (default: 20,000)

**File Filtering**

- Filter by pattern (`-f`, `--filter`)
- Case-sensitive filtering (`-c`, `--case-sensitive`)
- Support for complex filter patterns:
  - Simple text matching
  - Regular expressions
  - Glob patterns
  - Logical operators (AND, OR, NOT, XOR)

**Plugin System**

- Enable/disable plugins (`--enable-plugin`, `--disable-plugin`)
- Custom plugin directory support (`--plugins-dir`)
- Plugin action support (`--plugin-arg`)

## Usage

### Command Reference

**Basic Usage**

```bash
lla                     # List current directory (default view)
lla /path/to/dir       # List specific directory
lla -l                 # Long format with detailed information
lla -t                 # Tree view
lla -T                 # Table view
lla -g                 # Grid view
lla -G                 # Git-aware view
lla -S                 # Size map view
lla --timeline         # Timeline view
lla --icons           # Show file/directory icons
```

**Sorting & Organization**

```bash
lla -s name            # Sort by name (default)
lla -s size            # Sort by size
lla -s date            # Sort by date
lla -r                 # Reverse sort order
lla --sort-dirs-first  # List directories before files
lla --sort-case-sensitive # Case-sensitive sorting
lla --sort-natural     # Natural number sorting (2.txt before 10.txt)
```

**Depth Control**

```bash
lla -d 2               # List directory tree with depth 2
lla -t -d 3           # Tree view with max depth 3
```

### Filter System

#### Basic Filtering

```bash
lla -f "test"              # Find files containing "test"
lla -f "test" -c          # Case-sensitive search
lla -f ".rs"              # Find files with .rs extension
```

#### Advanced Filters

**Pattern Filters**

```bash
lla -f "test,spec"         # OR operation
lla -f "+test,api"         # AND operation
```

**Regular Expression Filters**

```bash
lla -f "regex:^test.*\.rs$"   # Rust files starting with "test"
lla -f "regex:\d{4}"          # Files containing 4 digits
```

**Glob Pattern Filters**

```bash
lla -f "glob:*.{rs,toml}"     # Match .rs or .toml files
lla -f "glob:test_*"          # Files starting with test_
```

#### Composite Filters

```bash
lla -f "test AND .rs"         # AND operation
lla -f "test OR spec"         # OR operation
lla -f "NOT test"             # NOT operation
lla -f "test XOR spec"        # XOR operation
```

### Plugin System

https://github.com/user-attachments/assets/6064b70e-1275-4d60-88ed-3370c0068ebd

**Installation**

```bash
# From Git repository
lla install --git https://github.com/user/plugin

# From local directory
lla install --dir path/to/plugin
```

**Management**

You can use the following commands to manage plugins:

```bash
lla use                    # Interactive plugin manager
lla --enable-plugin name   # Enable plugin
lla --disable-plugin name  # Disable plugin
lla update                 # Update all plugins
# You also update a single plugin
lla update file_tagger
```

**Plugin Actions:**

Plugin actions are the functions which a given plugin can perform.

```bash
lla plugin --name file_tagger --action add-tag --args README.md "important"
```

**Creating Shortcuts**

Shortcuts allow you to save frequently used plugin commands with simpler aliases:

```bash
# Add a shortcut
lla shortcut add find file_finder search -d "Quick file search"
lla shortcut add hash file_hash calculate -d "Calculate file hashes"
lla shortcut add todos keyword_search find-todos -d "Find TODO comments"

# List all shortcuts
lla shortcut list

# Use shortcuts (remaining arguments are passed to the plugin)
lla find pattern           # Equivalent to: lla plugin --name file_finder --action search --args "pattern"
lla hash filename         # Equivalent to: lla plugin --name file_hash --action calculate --args "filename"
lla todos                # Equivalent to: lla plugin --name keyword_search --action find-todos

# Remove a shortcut
lla shortcut remove find
```

## Configuration

The configuration file is located at `~/.config/lla/config.toml`. You can modify it directly or use the `lla config` command.

**Configuration File Format:**

```toml
# Core Settings
default_sort = "name"          # Possible values: "name", "size", "date"
default_format = "default"     # Possible values: "default", "long", "tree", "grid"
enabled_plugins = ["git_status", "file_hash"]  # List of enabled plugins
plugins_dir = "/home/user/.config/lla/plugins" # Plugin directory location
default_depth = 3              # Default depth for recursive listing

# Performance Tuning
[formatters.tree]
max_lines = 20000             # Maximum entries in tree view
                             # Set to 0 for unlimited (may impact performance)

[listers.recursive]
max_entries = 20000          # Maximum entries in recursive listing
                             # Set to 0 for unlimited (may impact performance)

# Command Shortcuts
[shortcuts]
find = { plugin_name = "finder", action = "search", description = "Quick file search" }
```

**Configuration Commands:**

```bash
# Initialize config
lla init                  # Create default config file

# View current config
lla config               # Display current configuration

# Modify settings
lla config --set default_sort size
lla config --set default_format long
lla config --set plugins_dir /custom/path
lla config --set default_depth 5

# Manage shortcuts
lla shortcut add NAME PLUGIN ACTION [-d DESCRIPTION]  # Add shortcut
lla shortcut remove NAME                             # Remove shortcut
lla shortcut list                                    # List all shortcuts
```

CLI Configuration:

```bash
lla config --set default_sort size
lla config --set default_format long
```

## Development

### Plugin Development

Develop custom plugins using the `Plugin` trait from [lla_plugin_interface](https://github.com/triyanox/lla/tree/main/lla_plugin_interface). More details in the [plugin development guide](https://github.com/triyanox/lla/tree/main/lla_plugin_interface).

### Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/new-feature`)
3. Commit your changes (`git commit -m 'Add some new-feature'`)
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
