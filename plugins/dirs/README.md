# LLA Directory Summary Plugin

A plugin for `lla` that provides quick directory statistics including file counts, sizes, and modification times with intelligent caching for better performance.

## What it Does

- Calculates directory statistics:
  - Total number of files
  - Number of subdirectories
  - Total size of all files
  - Last modification time
- Uses parallel processing for faster analysis
- Caches results for improved performance
- Automatically updates when directory contents change
- Provides human-readable size formatting

## Display Formats

### Default View

Shows basic directory information:

```
Documents (15 files, 2.5 GB)
```

### Detailed View (`-l` flag)

Shows complete directory information:

```
Documents (15 files, 3 dirs, 2.5 GB, modified 5 mins ago)
```

### Size Units

- Automatically adjusts units based on size:
  - Bytes (B) for < 1 KB
  - Kilobytes (KB) for < 1 MB
  - Megabytes (MB) for < 1 GB
  - Gigabytes (GB) for ≥ 1 GB

### Time Display

- Shows modification time in human-readable format:
  - Seconds for < 1 minute
  - Minutes for < 1 hour
  - Hours for < 1 day
  - Days for ≥ 1 day

The plugin automatically integrates with `lla`'s display system.
