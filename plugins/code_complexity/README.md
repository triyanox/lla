# LLA Code Complexity Plugin

A code analysis plugin for `lla` that estimates code complexity using various metrics and provides detailed reports for Rust, Python, and JavaScript/TypeScript files.

## What it Does

- Calculates code complexity metrics including:
  - Cyclomatic complexity
  - Cognitive complexity
  - Maintainability index (0-100)
  - Lines of code, functions, classes, branches, loops
  - Comment density and long lines detection
- Provides color-coded complexity indicators
- Generates detailed reports per language and file
- Supports configurable complexity thresholds

## Usage

1. Generate Report:

```bash
lla plugin --name code_complexity --action show-report
```

2. Configure Thresholds:

```bash
lla plugin --name code_complexity --action set-thresholds --args "10" "20" "30" "40"
```

3. View Help:

```bash
lla plugin --name code_complexity --action help
```

### Output Examples

Basic view:

```
[Complexity: 12 (MI: 85.3)] example.rs
```

Detailed view (`-l` flag):

```
[Complexity: 12 (MI: 85.3)]
  Lines: 150
  Functions: 5
  Classes: 2
  Branches: 8
  Loops: 4
  Comments: 20
  Long lines: 3
  Long functions:
    process_data (55 lines)
    analyze_results (60 lines)
```

Configuration is stored in `~/.config/lla/code_complexity.toml` and can be customized for each supported language.
