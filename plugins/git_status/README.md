# LLA Git Status Plugin

A plugin for `lla` that shows Git repository status with color-coded symbols and detailed information.

## What it Does

- Shows Git status for files and directories:
  - Staged, modified, and untracked files
  - Branch and commit information
  - Repository state (clean/changes)
  - Merge conflicts
- Uses color-coded symbols for quick status recognition
- Provides both simple and detailed views
- Automatically detects Git repositories

## Status Symbols

| Symbol | Meaning   | Color   |
| ------ | --------- | ------- |
| ✓      | Staged    | Green   |
| ±      | Modified  | Yellow  |
| ✚      | New file  | Green   |
| ✖      | Deleted   | Red     |
| ➜      | Renamed   | Purple  |
| ↠      | Copied    | Cyan    |
| ⚡     | Conflict  | Magenta |
| ?      | Untracked | Blue    |
| ⎇      | Branch    | Blue    |

## Display Formats

### Default View

Shows file status with symbols:

```
document.txt [✓ staged]
script.py [± modified]
new.rs [? untracked]
```

### Detailed View (`-l` flag)

Shows repository information:

```
project/
Branch:   ⎇ main
Commit:   a1b2c3d Initial commit
Status:   2 staged, 1 modified, 3 untracked
Repo:     has changes
```

The plugin automatically integrates with `lla`'s display system.
