# LLA File Tagger Plugin

A plugin for `lla` that helps you organize files using custom tags with persistent storage.

## What it Does

- Manages custom tags for files:
  - Add tags to files
  - Remove tags from files
  - List tags for files
- Displays tags next to file names
- Stores tags persistently
- Shows tags in cyan color for visibility

## Usage

### Basic Commands

```bash
# Add a tag
lla plugin --name file_tagger --action add-tag --args "file.txt" "important"

# Remove a tag
lla plugin --name file_tagger --action remove-tag --args "file.txt" "important"

# List tags
lla plugin --name file_tagger --action list-tags --args "file.txt"

# View help
lla plugin --name file_tagger --action help
```

### Display Format

Tags appear in cyan brackets next to file names:

```
document.pdf [important, work]
image.jpg [personal]
```

The plugin stores tags in `~/.config/lla/file_tags.txt` and automatically integrates with `lla`'s display system.
