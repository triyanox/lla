# LLA Categorizer Plugin

A file categorization plugin for `lla` that automatically categorizes files based on their extensions and provides detailed statistics about file categories in your directories.

## Features

- Automatic file categorization with colored output
- Category statistics with file counts and sizes
- Subcategory support for organization
- Configurable categories and rules

## Usage

The following actions are available through the plugin interface:

```bash
# Add a new category
lla plugin --name categorizer --action add-category --args "Audio" "yellow" "mp3,wav,flac,ogg" "Audio files"

# Add a subcategory to an existing category
lla plugin --name categorizer --action add-subcategory --args "Audio" "Lossless" "flac,wav"

# Show statistics about file categories
lla plugin --name categorizer --action show-stats

# List all configured categories
lla plugin --name categorizer --action list-categories

# Show help and available actions
lla plugin --name categorizer --action help
```

## Configuration

The plugin stores its configuration in `~/.config/lla/categorizer.toml`. The configuration includes:

- Category names and colors
- File extensions for each category
- Subcategories and their extensions
- Optional size ranges for categorization

Default categories include Documents, Images, and Code, each with their own subcategories and extensions.
