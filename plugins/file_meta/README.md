# LLA File Metadata Plugin

A plugin for `lla` that displays detailed file metadata including timestamps, ownership, size, and permissions.

## What it Does

- Shows comprehensive file metadata:
  - Access, modification, and creation timestamps
  - User and group IDs (UID/GID)
  - File size in bytes
  - File permissions in octal format
- Formats timestamps in human-readable format (YYYY-MM-DD HH:MM:SS)
- Provides color-coded output for better readability

## Display Format

Shows metadata information below each file:

```
document.pdf
Accessed: 2024-03-15 14:30:22
Modified: 2024-03-15 14:30:20
Created:  2024-03-15 14:30:18
UID/GID:  1000/1000
Size:     1048576
Perms:    644
```

### Color Coding

- Access Time: Blue
- Modification Time: Green
- Creation Time: Yellow
- UID/GID: Magenta
- File Size: Cyan
- Permissions: Red

The plugin automatically integrates with `lla`'s display system and works in both default and long view formats.
