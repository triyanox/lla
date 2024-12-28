# LLA Last Git Commit Plugin

Git history plugin for `lla` providing real-time commit tracking with rich formatting.

## Features

- Short hash, author, and timestamp display
- Path-specific history
- Color-coded information
- Multiple display formats
- Smart caching and quick lookups

## Configuration

Config file: `~/.config/lla/last_git_commit/config.toml`

```toml
[colors]
hash = "bright_yellow"
author = "bright_cyan"
time = "bright_green"
info = "bright_blue"
name = "bright_yellow"
```

## Display Formats

Default:

```
document.txt
Commit: a1b2c3d 2 days ago
```

Long:

```
document.txt
Commit:  a1b2c3d
Author:  John Doe
Time:    2 days ago
```
