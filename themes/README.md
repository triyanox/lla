# LLA Themes

LLA provides a powerful and flexible theming system that allows you to customize the appearance of your file listings. Each theme is defined in a TOML file and can customize colors for files, folders, permissions, and more.

## Table of Contents

- [Theme Location](#theme-location)
- [Theme Structure](#theme-structure)
- [Color Formats](#color-formats)
- [Basic Elements](#basic-elements)
- [Special Files](#special-files)
- [Extension System](#extension-system)
- [Built-in Themes](#built-in-themes)
- [Usage](#usage)

## Theme Location

Themes are stored in `~/.config/lla/themes/` with a `.toml` extension:

```
~/.config/lla/themes/
├── dark.toml
├── light.toml
├── poimandres.toml
└── custom.toml
```

## Theme Structure

A theme file has the following main sections:

```toml
# Theme metadata
name = "my_theme"
author = "Your Name"
description = "A description of your theme"

# Core colors for basic elements
[colors]
file = "#FFFFFF"
directory = "#89AFFF"
...

# Special file rules
[special_files]
...

# Extension-based colors
[extensions]
...
```

## Color Formats

LLA supports multiple color formats for maximum flexibility:

```toml
[colors]
# 1. Named Colors (Basic terminal colors)
file = "white"
directory = "blue"
symlink = "bright_cyan"

# 2. RGB Format
file = { r = 255, g = 255, b = 255 }
directory = { r = 81, g = 154, b = 255 }

# 3. Hex Format
file = "#FFFFFF"              # 6-digit hex
directory = "#89F"           # 3-digit hex
symlink = "#89DBFFFF"       # 8-digit hex (alpha ignored)

# 4. HSL Format (h: 0-360, s: 0-1, l: 0-1)
directory = { h = 180, s = 0.5, l = 0.6 }

# 5. No Color
disabled = "none"
```

### Available Named Colors

Basic Colors:

- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`

Bright Variants:

- `bright_black` (or `gray`/`grey`)
- `bright_red`, `bright_green`, `bright_yellow`
- `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`

Additional Colors:

- `navy`, `teal`, `maroon`, `purple`, `olive`, `silver`

## Basic Elements

The `[colors]` section defines the core colors:

```toml
[colors]
# Core UI Elements
file = "#FFFFFF"        # Regular files
directory = "#89AFFF"   # Directories
symlink = "#89DBFF"     # Symbolic links
executable = "#5DE4B3"  # Executable files

# Metadata
size = "#5DE4B3"       # File sizes
date = "#FCBCFA"       # Timestamps
user = "#D0679D"       # User names
group = "#8A9092"      # Group names

# Permissions
permission_dir = "#89AFFF"   # Directory permission
permission_read = "#5DE4B3"  # Read permission
permission_write = "#FFFFC2" # Write permission
permission_exec = "#D0679D"  # Execute permission
permission_none = "#282E30"  # No permission
```

## Special Files

The `[special_files]` section allows you to define custom colors for specific files and patterns:

### 1. Custom Folders

```toml
[special_files.folders]
# Exact folder names
"node_modules" = "#666666"  # Dim gray for node modules
"target" = "#444444"        # Dark gray for build outputs
".git" = "#FF6B6B"          # Red for git directory

# Folder patterns
"build*" = "#444444"        # Any folder starting with "build"
"*.cache" = "#666666"       # Any folder ending with ".cache"
"*-env" = "#5DE4B3"        # Any folder ending with "-env"
```

### 2. Dotfiles

```toml
[special_files.dotfiles]
".gitignore" = "#89DBFF"
".env" = "#FFFFC2"
".dockerignore" = "#89DBFF"
```

### 3. Exact Matches

```toml
[special_files.exact_match]
"Dockerfile" = "#89DBFF"
"Makefile" = "#D0679D"
"README.md" = "#FFFFC2"
```

### 4. Pattern Matches

```toml
[special_files.patterns]
"*rc" = "#89DBFF"         # Files ending in "rc"
"*.min.*" = "#282E30"     # Minified files
"*.test.*" = "#5DE4B3"    # Test files
```

## Extension System

The extension system provides two ways to define colors for file extensions:

### 1. Extension Groups

Group related extensions together:

```toml
[extensions.groups]
# Group multiple extensions
rust = ["rs", "toml"]
web = ["html", "css", "js"]
docs = ["md", "rst", "txt"]

# Define colors for groups
[extensions.colors]
rust = "#FF5733"    # All rust-related files
web = "#61AFEF"     # All web files
docs = "#98C379"    # All documentation files

# Override specific extensions
rs = "#FF0000"      # Override just .rs files
```

## Color Resolution Order

When determining a file's color, LLA follows this priority:

1. If it's a directory:

   - Exact folder name match
   - Folder pattern match
   - Default directory color

2. If it's a file:
   - Exact filename match
   - Dotfile match
   - Pattern match
   - Extension-specific color
   - Extension group color
   - Default file color

## Built-in Themes

LLA includes several pre-configured themes:

- **default**: Traditional terminal colors with carefully selected colors for optimal visibility
- **dark**: Modern dark theme with enhanced visibility and rich colors for dark terminals
- **light**: Clean theme optimized for light terminals with maximum readability
- **ayu_dark**: Dark theme inspired by the Ayu Dark color scheme, featuring deep backgrounds and vibrant accents
- **ayu_light**: Light theme with carefully selected colors for optimal readability
- **ayu_mirage**: Refined dark theme with muted colors and soft contrasts
- **catppuccin_mocha**: A soothing pastel theme for the high-spirited, featuring warm, cozy colors
- **dracula**: Classic Dracula color scheme with vibrant colors and high contrast
- **gruvbox_dark**: Retro groove color scheme with warm, earthy tones and high contrast
- **material_ocean**: Deep blue theme based on Material Design, featuring oceanic colors
- **nord**: Arctic, north-bluish color palette with elegant pastel colors
- **one_dark**: Dark theme inspired by Atom, featuring a perfect balance of cool and warm colors
- **poimandres**: Deep space aesthetic with vibrant accents
- **tokyo_night**: Dark theme inspired by the vibrant lights of Tokyo at night
- **vesper**: Minimalist dark theme with warm accents

## Usage

Set your theme in `~/.config/lla/config.toml`:

```toml
theme = "dark"  # Without .toml extension
```

Or use the command line:

```bash
lla config --set theme dark
```

To disable colors:

```bash
lla config --set theme none
```

or if you don't want any colors you can add "--no-colors" flag to lla:

```bash
lla --no-colors # works with all listing commands
```
