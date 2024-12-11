# LLA File Hash Plugin

A plugin for `lla` that calculates and displays SHA-1 and SHA-256 hashes for files.

## What it Does

- Calculates two types of hashes for each file:
  - SHA-1 (shown in green)
  - SHA-256 (shown in yellow)
- Shows first 8 characters of each hash
- Uses efficient buffered reading
- Skips directories automatically
- Displays hashes in a clean, formatted box layout

## Display Format

Shows hash information below each file:

```
document.pdf
┌ SHA1   → a1b2c3d4
└ SHA256 → e5f6g7h8
```

### Color Coding

- SHA-1: Green text
- SHA-256: Yellow text
- Box characters and arrows: Dark gray
- Hash names: Bold colored text

The plugin automatically integrates with `lla`'s display system and works in both default and long view formats.

Note: The truncated 8-character display is for readability. For security-critical comparisons, use a dedicated hashing tool.
