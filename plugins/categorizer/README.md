# LLA Categorizer Plugin

A powerful file categorization plugin for `lla` that automatically organizes and labels files based on their extensions, with support for hierarchical categorization and detailed metadata tracking.

## Features

- **Smart File Categorization**: Automatically categorizes files based on extensions with colored labels
- **Hierarchical Organization**: Support for categories and subcategories
- **Size-Based Rules**: Optional file size ranges for more precise categorization
- **Rich Formatting**: Two display formats (default and long) with colored output
- **Statistics Tracking**: Maintains counts and size statistics for categories and subcategories
- **Fully Configurable**: Easy to add and customize categories, colors, and rules

## Default Categories

The plugin comes with pre-configured categories:

### Documents

- Color: bright_blue
- Extensions: txt, md, doc, docx, pdf, rtf, odt
- Size Range: 0-10MB
- Subcategories:
  - Text: txt, md
  - Office: doc, docx, xls, xlsx, ppt, pptx

### Code

- Color: bright_cyan
- Extensions: rs, py, js, ts, java, c, cpp, h, hpp, go, rb, php, cs, swift, kt
- Size Range: 0-1MB
- Subcategories:
  - Systems: rs, c, cpp, h, hpp
  - Web: js, ts, html, css, php
  - Scripts: py, rb, sh, bash

## Usage

### Adding Categories

```bash
# Add a new category
lla plugin --name categorizer --action add-category "Images" "yellow" "jpg,png,gif" "Image files"
```

### Managing Subcategories

```bash
# Add a subcategory to an existing category
lla plugin --name categorizer --action add-subcategory "Images" "Raster" "jpg,png,gif"
```

### Viewing Categories

```bash
# List all configured categories and their details
lla plugin --name categorizer --action list-categories

# Show help and available commands
lla plugin --name categorizer --action help
```

## Configuration

The plugin configuration is stored in `~/.config/lla/plugins/categorizer/config.toml` and includes:

- Category definitions (name, color, description)
- File extension mappings
- Size range rules
- Subcategory configurations
- UI color schemes

The configuration is automatically created with sensible defaults on first run and can be customized as needed.

## Display Formats

The plugin supports two display formats:

- **default**: Shows category in colored brackets (e.g., `[Documents]`)
- **long**: Shows category with subcategory (e.g., `[Documents] (Text)`)

## Development

This plugin is built using Rust and integrates with the LLA plugin system. It uses:

- `serde` for configuration serialization
- `colored` for terminal output
- `parking_lot` for thread-safe state management
- `toml` for configuration file handling
