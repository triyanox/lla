# LLA Keyword Search Plugin

A plugin for `lla` that searches for keywords in files with configurable context and highlighting.

## What it Does

- Searches for configured keywords in files
- Shows matches with surrounding context lines
- Supports multiple search options:
  - Case-sensitive search
  - Multiple keywords
  - Configurable context lines
  - Maximum matches per file
- Highlights matches in color
- Supports many file extensions (txt, md, rs, py, js, etc.)

## Usage

### Basic Commands

```bash
# Set keywords to search for
lla plugin --name keyword_search --action set-keywords --args "TODO" "FIXME"

# Search in a specific file
lla plugin --name keyword_search --action search --args "src/main.rs"

# Show current configuration
lla plugin --name keyword_search --action show-config

# View help
lla plugin --name keyword_search --action help
```

### Configuration Commands

```bash
# Set case sensitivity
lla plugin --name keyword_search --action set-case-sensitive --args true

# Set number of context lines
lla plugin --name keyword_search --action set-context-lines --args 3

# Set maximum matches per file
lla plugin --name keyword_search --action set-max-matches --args 5
```

### Display Format

Default view shows line numbers and matches:

```
15:TODO - Add error handling here
42:FIXME - Need to optimize this loop
```

Detailed view (`-l` flag) shows context:

```
  12: function processData() {
  13:   let data = [];
→ 14: // TODO: Add error handling here
  15:   return data;
  16: }
```

The plugin stores configuration in `~/.config/lla/plugins/keyword_search.toml` and automatically integrates with `lla`'s display system.
