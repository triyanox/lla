# LLA Code Snippet Extractor Plugin

A plugin for `lla` that helps you extract, organize, and manage code snippets from your files with tagging and search capabilities.

## What it Does

- Extracts code snippets with customizable context lines
- Automatically detects language based on file extension
- Organizes snippets with tags
- Provides full-text search across all snippets
- Preserves code context (before/after the snippet)
- Supports import/export of snippets
- Tracks snippet versions and modifications

## Usage

### Basic Operations

```bash
# Extract a snippet (lines 10-20 with 3 context lines)
lla plugin --name code_snippet_extractor --action extract --args "file.rs" "function_name" 10 20 3

# List snippets in a file
lla plugin --name code_snippet_extractor --action list --args "file.rs"

# View a specific snippet
lla plugin --name code_snippet_extractor --action get --args "file.rs" "function_name"
```

### Organization

```bash
# Search snippets
lla plugin --name code_snippet_extractor --action search --args "query"

# Add tags
lla plugin --name code_snippet_extractor --action add-tags --args "file.rs" "function_name" "tag1" "tag2"

# Remove tags
lla plugin --name code_snippet_extractor --action remove-tags --args "file.rs" "function_name" "tag1"
```

### Import/Export

```bash
# Export snippets
lla plugin --name code_snippet_extractor --action export --args "file.rs"

# Import snippets
lla plugin --name code_snippet_extractor --action import --args "file.rs" "toml_data"
```

### View Help

```bash
lla plugin --name code_snippet_extractor --action help
```

### Output Example

```
┌─ Context Before ─────────────
// Helper function for parsing

├─ Snippet Content ──────────────
fn parse_input<T: FromStr>(input: &str) -> Option<T> {
    input.trim().parse().ok()
}

├─ Context After ────────────────
// Example usage:
// let num: i32 = parse_input("42").unwrap();

├─ Metadata ────────────────────
│ Language: rust
│ Version: 1
│ Tags: #parser #input
└──────────────────────────────
```

Snippets are stored in `~/.config/lla/code_snippets.toml` and can be backed up or version controlled.
