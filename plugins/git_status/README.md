# LLA Git Status Plugin

Git integration plugin for `lla` providing real-time repository status with rich formatting.

## Features

- Status tracking (staged, modified, untracked, conflicts)
- Repository info (branch, commits, working tree)
- Color-coded display
- Performance optimized

## Configuration

Config location: `~/.config/lla/git_status/config.toml`

```toml
[colors]
clean = "bright_green"
modified = "bright_yellow"
staged = "bright_green"
untracked = "bright_blue"
conflict = "bright_red"
branch = "bright_cyan"
commit = "bright_yellow"
info = "bright_blue"
name = "bright_yellow"
```

## Display Examples

Basic:

```
document.txt
Git: modified, 2 staged
```

Detailed:

```
project/
Branch:  main
Commit:  a1b2c3d Initial commit
Status:  2 staged, 1 modified, 3 untracked
```
