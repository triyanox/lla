# LLA Size Visualizer Plugin

A plugin for `lla` that visualizes file sizes using colored bars and human-readable formats.

## What it Does

- Shows file sizes in two ways:
  - Visual bar representation
  - Human-readable size format (B, KB, MB, GB, TB)
- Uses color coding based on size ranges:
  - Green: ≤ 1KB
  - Bright Green: 1KB - 10KB
  - Cyan: 10KB - 1MB
  - Blue: 1MB - 10MB
  - Yellow: 10MB - 100MB
  - Red: 100MB - 1GB
  - Magenta: > 1GB
- Shows percentage relative to 1GB reference
- Provides both compact and detailed views

## Display Formats

### Default View

Shows a compact size visualization:

```
file.txt █████     2.5 MB
```

### Detailed View (`-l` flag)

Shows complete size information:

```
┌─ Size ────────────────────────────────────
│ █░░░░░░░░░░░░░░░░░░ 25.5 MB
│ 2.5% of reference (1GB)
└──────────────────────────────────────────
```

The plugin automatically integrates with `lla`'s display system.
