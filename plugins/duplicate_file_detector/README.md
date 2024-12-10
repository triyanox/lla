# LLA Duplicate File Detector Plugin

A plugin for `lla` that identifies duplicate files by comparing their content using SHA-256 hashing.

## What it Does

- Detects duplicate files by comparing file contents
- Identifies original files and their duplicates
- Tracks file modification times to determine originals
- Uses SHA-256 hashing for reliable comparison
- Caches results for better performance

## Display Formats

### Default View

Shows basic duplicate information:

```
file.txt (DUPLICATE)
other.txt (HAS DUPLICATES)
```

### Detailed View (`-l` flag)

Shows complete duplicate information:

```
file.txt (DUPLICATE of: /path/to/original.txt)
other.txt (HAS DUPLICATES copies: /path/to/copy1.txt, /path/to/copy2.txt)
```

### Color Coding

- Original files with duplicates: Yellow
- Duplicate files: Red
- File paths: Cyan

The plugin automatically integrates with `lla`'s display system.
