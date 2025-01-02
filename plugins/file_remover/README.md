# LLA File Remover Plugin

A plugin for `lla` that provides an interactive interface for safely removing files and directories.

## Features

- **Interactive Selection**: Multi-select interface for choosing files to remove
- **Path Flexibility**: Support for both current and specified directories
- **Safe Operations**: Confirmation prompts and error handling for safe removal
- **Directory Support**: Recursive removal of directories
- **User Interface**: Colored output and interactive menus

## Configuration

```toml
[colors]
success = "bright_green"
info = "bright_blue"
error = "bright_red"
path = "bright_yellow"
```

## Usage

### Basic Operations

```bash
# Remove files/directories from current directory
lla plugin --name file_remover --action remove

# Remove files/directories from specified directory
lla plugin --name file_remover --action remove /path/to/directory

# Show help information
lla plugin --name file_remover --action help
```

## Common Workflows

### 1. Removing Files from Current Directory

```bash
# In target directory
cd /path/to/directory
lla plugin --name file_remover --action remove
# Select files to remove using space, confirm with enter
```

### 2. Removing Files from Specific Directory

```bash
# Remove files from a specific directory without changing location
lla plugin --name file_remover --action remove /path/to/directory
# Select files to remove using space, confirm with enter
```

## Display Format

```
─────────────────────────────────────
 File Remover
─────────────────────────────────────
 Select items to remove:
   → file1.txt
   → directory1
   → file2.rs
─────────────────────────────────────
 Use Space to select, Enter to confirm
─────────────────────────────────────
```
