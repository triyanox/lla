# LLA Size Visualizer Plugin

File size visualization plugin for `lla` providing real-time size analysis with rich formatting.

## Features

- Human-readable size formatting
- Visual progress bars with Unicode blocks
- Size-based color coding
- Multiple display formats
- Smart caching and quick analysis

## Configuration

Config file: `~/.config/lla/sizeviz/config.toml`

```toml
[colors]
tiny = "bright_green"      # ≤ 1KB
small = "bright_cyan"      # 1KB - 1MB
medium = "bright_yellow"   # 1MB - 10MB
large = "bright_red"       # 10MB - 100MB
huge = "bright_magenta"    # > 100MB
info = "bright_blue"
size = "bright_yellow"
percentage = "bright_magenta"
```

## Display Formats

Default:

```
document.pdf
█████░░░░░ 2.5 MB
```

Long:

```
┌─ Size ────────────────────────────────────
│ ████████████░░░░░░░░░░ 25.5 MB
│ 2.5% of reference (1GB)
└──────────────────────────────────────────
```
