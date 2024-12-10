# LLA Last Git Commit Plugin

A plugin for `lla` that shows the last Git commit information for files with colored output.

## What it Does

- Shows last commit information for files:
  - Commit hash (short version)
  - Author name (in detailed view)
  - Time since commit
- Uses color coding for better readability:
  - Hash in yellow
  - Author in cyan
  - Time in green
- Automatically detects Git repositories
- Works with both files and directories

## Display Formats

### Default View

Shows basic commit information:

```
document.txt [Commit: a1b2c3d 2 days ago]
```

### Detailed View (`-l` flag)

Shows complete commit information:

```
document.txt [Last commit: a1b2c3d by John Doe 2 days ago]
```

The plugin automatically integrates with `lla`'s display system.
