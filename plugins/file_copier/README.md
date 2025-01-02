# LLA File Copier Plugin

A plugin for `lla` that provides an intuitive clipboard-based interface for copying files and directories.

## Features

- **Clipboard Management**: Persistent clipboard for files and directories
- **Interactive Selection**: Multi-select interface for files and operations
- **Flexible Copying**: Copy all or selected items from clipboard
- **Path Flexibility**: Support for both current and specified directories
- **Safe Operations**: Validation and error handling for copy operations
- **User Interface**: Colored output and interactive menus

## Configuration

Config file: `~/.config/lla/cp_clipboard.json`

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
# Add files from current directory to clipboard
lla plugin --name file_copier --action add

# Add files from a specific directory to clipboard
lla plugin --name file_copier --action add /path/to/source

# Copy all files from clipboard to current directory
lla plugin --name file_copier --action copy-all

# Copy all files from clipboard to specific directory
lla plugin --name file_copier --action copy-all /path/to/destination

# Copy selected files from clipboard to current directory
lla plugin --name file_copier --action copy-partial

# Copy selected files from clipboard to specific directory
lla plugin --name file_copier --action copy-partial /path/to/destination
```

### Clipboard Management

```bash
# View clipboard contents with option to remove items
lla plugin --name file_copier --action show

# Clear the clipboard
lla plugin --name file_copier --action clear

# Show help information
lla plugin --name file_copier --action help
```

## Common Workflows

### 1. Copying Files Between Directories (Using Explicit Paths)

```bash
# Add files from source directory
lla plugin --name file_copier --action add /path/to/source
# Select files to copy using space, confirm with enter

# Copy all files to target directory
lla plugin --name file_copier --action copy-all /path/to/target
```

### 2. Copying Files Using Current Directory Navigation

```bash
# In source directory
cd /path/to/source
lla plugin --name file_copier --action add
# Select files to add to clipboard

# Navigate to first target
cd /path/to/target1
lla plugin --name file_copier --action copy-partial
# Select subset of files to copy here

# Navigate to second target
cd /path/to/target2
lla plugin --name file_copier --action copy-partial
# Select another subset of files to copy here
```

### 3. Mixed Workflow (Current and Explicit Paths)

```bash
# Add files from current directory
lla plugin --name file_copier --action add
# Select files to add to clipboard

# Copy selected files to a specific directory without changing location
lla plugin --name file_copier --action copy-partial /path/to/target
```

## Display Format

```
─────────────────────────────────────
 File Copier Clipboard
─────────────────────────────────────
 Current Items:
   → /path/to/file1.txt
   → /path/to/directory1
   → /path/to/file2.rs
─────────────────────────────────────
 Use Space to select, Enter to confirm
─────────────────────────────────────
```

## Features in Detail

1. **Clipboard Persistence**

   - Clipboard state persists between sessions
   - Stored in user config directory
   - JSON format for easy debugging

2. **Interactive Selection**

   - Multi-select interface for file operations
   - Clear visual feedback
   - Easy navigation with keyboard

3. **Path Flexibility**

   - Support for current directory operations
   - Support for explicit path specifications
   - Mix and match approaches in the same workflow

4. **Safe Operations**

   - Validation before copying files
   - Error handling for common scenarios
   - Clear feedback on operation status

5. **Flexible Workflows**
   - Support for multiple target directories
   - Partial copies from clipboard
   - Easy clipboard management
