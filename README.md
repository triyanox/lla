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
- [Theming](#theming)
- [Completion](#completion)
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

# Manual - Example is for amd64 GNU, replaces the file names if downloading for a different arch.
wget -c https://github.com/triyanox/lla/releases/download/v0.3.6/lla-linux-amd64 -O lla
sudo chmod +x lla
sudo chown root:root lla
sudo mv lla /usr/local/bin/lla
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

![default](https://github.com/user-attachments/assets/ba5fa273-c2c4-4143-b199-ab5bff1bb608)

#### Long Format

Detailed file information with metadata

```bash
lla -l
```

![long](https://github.com/user-attachments/assets/c736720c-cd00-453e-b5b3-5cdf91fd60d1)

#### Tree View

Hierarchical directory visualization

```bash
lla -t
```

![tree](https://github.com/user-attachments/assets/e8593ec5-5566-44ba-bbe6-712ebd1656ec)

#### Table View

Structured data display

```bash
lla -T
```

![table](https://github.com/user-attachments/assets/707a76e3-fdda-4de1-9733-5bc2a05b80e0)

#### Grid View

Organized layout for better readability

```bash
lla -g
```

![grid](https://github.com/user-attachments/assets/68af78a6-277c-4942-ba46-068c04442fa4)

#### Git-Aware View

Repository status and insights

```bash
lla -G
```

![git](https://github.com/user-attachments/assets/127b8b6b-47d8-4fa5-95d6-d0217c253b72)

#### Timeline View

Group files by dates

```bash
lla --timeline
```

![timeline](https://github.com/user-attachments/assets/a5cd7579-6235-4757-a2b5-1a61735093fd)

#### Sizemap View

Visualize file sizes relative to each other

```bash
lla -S
```

![sizemap](https://github.com/user-attachments/assets/07cdf955-3f8d-4ae7-bbd0-ce97d7472967)

#### Fuzzy Search (Experimental)

Search for files using fuzzy matching

```bash
lla --fuzzy
```

![fuzzy](https://github.com/user-attachments/assets/98bf579c-ce71-4a4f-b0fc-bdb769d9cfe9)

#### Recursive Format

List all files in a directory recursively

```bash
lla -R # use -d to control the depth
```

![recursive](https://github.com/user-attachments/assets/6ba6b968-8fe3-475c-84c5-debdee2f97f5)

### Core Features

**Display Options**

- Long format (`-l`, `--long`): Detailed file information with metadata
- Tree view (`-t`, `--tree`): Hierarchical directory visualization
- Table view (`-T`, `--table`): Structured data display
- Grid view (`-g`, `--grid`): Organized layout for better readability
- Size map (`-S`, `--sizemap`): Visual representation of file sizes
- Timeline view (`--timeline`): Group files by time periods
- Git-aware view (`-G`, `--git`): Repository status and information
- Fuzzy search (`--fuzzy`): Search for files using fuzzy matching (Experimental)
- Recursive format (`-R`, `--recursive`): List all files in a directory recursively
- Icons (`--icons`): Show icons for files and directories
- No colors (`--no-colors`): Disable colors in all listing commands
- No icons (`--no-icons`): Disable icons in all listing commands
- Include directories (`--include-dirs`): Include the size of directories in the metadata, it will recursively calculate the size of all directories, works with all listing commands

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

- Show only specific types:
  - `--dirs-only`: Show only directories
  - `--files-only`: Show only regular files
  - `--symlinks-only`: Show only symbolic links
  - `--dotfiles-only`: Show only dot files and directories (those starting with a dot)
- Hide specific types:
  - `--no-dirs`: Hide directories
  - `--no-files`: Hide regular files
  - `--no-symlinks`: Hide symbolic links
  - `--no-dotfiles`: Hide dot files and directories (those starting with a dot)

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
lla -S                 # Size map view - you can also use the `include_dirs` flag to include the size of directories
lla --timeline         # Timeline view
lla --icons           # Show file/directory icons
lla --fuzzy            # Fuzzy search (Experimental)
lla -R                 # Recursive format - use -d to control the depth
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

# Show only specific types
lla --dirs-only        # Show only directories
lla --files-only       # Show only regular files
lla --symlinks-only    # Show only symbolic links
lla --dotfiles-only    # Show only dot files and directories

# Hide specific types
lla --no-dirs          # Hide directories
lla --no-files         # Hide regular files
lla --no-symlinks      # Hide symbolic links
lla --no-dotfiles      # Hide dot files and directories

# Combine filters
lla --dirs-only --dotfiles-only  # Show only dot directories
lla --files-only --no-dotfiles   # Show only regular files, excluding dot files
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
# LLA Configuration File
# This file controls the behavior and appearance of the lla command

# Default sorting method for file listings
# Possible values:
#   - "name": Sort alphabetically by filename (default)
#   - "size": Sort by file size, largest first
#   - "date": Sort by modification time, newest first
default_sort = "name"

# Default format for displaying files
# Possible values:
#   - "default": Quick and clean directory listing
#   - "long": Detailed file information with metadata
#   - "tree": Hierarchical directory visualization
#   - "fuzzy": Interactive fuzzy search
#   - "grid": Organized grid layout for better readability
#   - "git": Git-aware view with repository status
#   - "timeline": Group files by time periods
#   - "sizemap": Visual representation of file sizes
#   - "table": Structured data display
default_format = "default"

# Whether to show icons by default
# When true, file and directory icons will be displayed in all views
# Default: false
show_icons = true

# Whether to include directory sizes in file listings
# When true, directory sizes will be calculated recursively
# This may impact performance for large directories
# Default: false
include_dirs = false

# The theme to use for coloring
# Place custom themes in ~/.config/lla/themes/
# Default: "default"
theme = "default"

# List of enabled plugins
# Each plugin provides additional functionality
# Examples:
#   - "git_status": Show Git repository information
#   - "file_hash": Calculate and display file hashes
#   - "file_tagger": Add and manage file tags
enabled_plugins = []

# Directory where plugins are stored
# Default: ~/.config/lla/plugins
plugins_dir = "~/.config/lla/plugins"

# Maximum depth for recursive directory traversal
# Controls how deep lla will go when showing directory contents
# Set to None for unlimited depth (may impact performance)
# Default: 3 levels deep
default_depth = 3

# Sorting configuration
[sort]
# List directories before files
# Default: false
dirs_first = false

# Enable case-sensitive sorting
# Default: false
case_sensitive = false

# Use natural sorting for numbers (e.g., 2.txt before 10.txt)
# Default: true
natural = true

# Filtering configuration
[filter]
# Enable case-sensitive filtering by default
# Default: false
case_sensitive = false

# Hide dot files and directories by default
# Default: false
no_dotfiles = false

# Formatter-specific configurations
[formatters.tree]
# Maximum number of entries to display in tree view
# Controls memory usage and performance for large directories
# Set to 0 to show all entries (may impact performance)
# Default: 20000 entries
max_lines = 20000

# Lister-specific configurations
[listers.recursive]
# Maximum number of entries to process in recursive listing
# Controls memory usage and performance for deep directory structures
# Set to 0 to process all entries (may impact performance)
# Default: 20000 entries
max_entries = 20000

# Fuzzy lister configuration
[listers.fuzzy]
# Patterns to ignore when listing files in fuzzy mode
# Can be:
#  - Simple substring match: "node_modules"
#  - Glob pattern: "glob:*.min.js"
#  - Regular expression: "regex:.*\\.pyc$"
# Default: ["node_modules", "target", ".git", ".idea", ".vscode"]
ignore_patterns = ["node_modules","target",".git",".idea",".vscode"]
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
lla config --set show_icons true
lla config --set sort.dirs_first true
lla config --set sort.case_sensitive true
lla config --set filter.case_sensitive true
lla config --set filter.no_dotfiles true   # Hide dot files by default

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

## Theming

LLA includes a powerful theming system that allows you to customize the appearance of your file listings. Themes are defined in TOML files and stored in `~/.config/lla/themes/`.

You can use the interactive theme manager to switch themes using the `lla theme` command:

```bash
lla theme
```

![themes](https://github.com/user-attachments/assets/23ef3b9c-626b-45d1-afcf-22d249c8e499)

**Theme Structure:**

```toml
# Theme metadata
name = "my_theme"
author = "Your Name"
description = "A description of your theme"

# Core colors
[colors]
file = "#FFFFFF"        # Regular files
directory = "#89AFFF"   # Directories
symlink = "#89DBFF"     # Symbolic links
executable = "#5DE4B3"  # Executable files

# Special files
[special_files]
folders."node_modules" = "#666666"
dotfiles.".env" = "#FFFFC2"
exact_match."README.md" = "#FFFFC2"
patterns."*.min.*" = "#282E30"

# Extension-based colors
[extensions.groups]
rust = ["rs", "toml"]
web = ["html", "css", "js"]

[extensions.colors]
rust = "#FF5733"
web = "#61AFEF"
```

**Built-in Themes:**

- **default**: Traditional terminal colors optimized for visibility
- **dark**: Modern dark theme with enhanced visibility
- **light**: Clean theme optimized for light terminals
- **ayu_dark**, **ayu_light**, **ayu_mirage**: Inspired by the Ayu color scheme
- **catppuccin_mocha**: Soothing pastel theme with warm colors
- **dracula**: Classic Dracula scheme with vibrant colors
- **gruvbox_dark**: Retro groove theme with earthy tones
- **material_ocean**: Deep blue Material Design theme
- **nord**: Arctic, north-bluish color palette
- **one_dark**: Atom-inspired balanced dark theme
- **poimandres**: Deep space aesthetic theme
- **tokyo_night**: Vibrant dark theme inspired by Tokyo nights
- **vesper**: Minimalist dark theme with warm accents

**Using Themes:**

Copy the theme you want to use to your `~/.config/lla/themes/` directory.

and then set the theme in your config or use the command line:

```bash
# Set theme in config
lla config --set theme dark

# Disable colors
lla config --set theme none
```

also you can disable colors all together by using the `--no-colors` flag:

```bash
lla --no-colors # works with all listing commands
```

For more detailed information about theming, see the [themes documentation](themes/README.md).

## Completion

LLA supports shell completion for bash, zsh, fish, and elvish.

```bash
lla completion bash # for bash
lla completion zsh # for zsh
lla completion fish # for fish
lla completion elvish # for elvish
```

You can also generate completions in a specific directory by running the following command:

```bash
lla completion <shell> --output /path/to/completion
```

All completions are included [here](https://github.com/triyanox/lla/tree/main/completions).

## Development

### Plugin Development

Develop custom plugins using the `Plugin` trait from [lla_plugin_interface](https://github.com/triyanox/lla/tree/main/lla_plugin_interface). More details in the [plugin development guide](https://github.com/triyanox/lla/tree/main/lla_plugin_interface).

### Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/new-feature`)
3. Commit your changes (`git commit -m 'Add some new-feature'`)
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

If you want to add a new theme, please add it to the `themes/` directory.

If you want to add a new plugin, please add it to the `plugins/` directory or you can use your own repo to host it.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
