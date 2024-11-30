# LLA Plugins

This document lists all available plugins for LLA and provides installation instructions.

## Installation

You can install all plugins at once using:

```bash
lla install --git https://github.com/triyanox/lla
```

Or you can install individual plugins as described below.

## Available Plugins

### file_tagger

**Description:** A plugin for tagging files and filtering by tags

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/file_tagger
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/file_tagger
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### last_git_commit

**Description:** A plugin for the LLA that provides the last git commit hash

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/last_git_commit
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/last_git_commit
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### code_snippet_extractor

**Description:** A plugin for extracting and managing code snippets

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/code_snippet_extractor
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/code_snippet_extractor
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### file_type

**Description:** Displays the file type of each file using emojis

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/file_type
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/file_type
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### code_complexity

**Description:** Code complexity plugin for LLA

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/code_complexity
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/code_complexity
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### sizeviz

**Description:** File size visualizer plugin for LLA

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/sizeviz
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/sizeviz
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### file_hash

**Description:** Displays the hash of each file

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/file_hash
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/file_hash
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### categorizer

**Description:** Categorizer plugin for LLA

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/categorizer
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/categorizer
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### duplicate_file_detector

**Description:** A plugin for the LLA that detects duplicate files.

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/duplicate_file_detector
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/duplicate_file_detector
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### keyword_searcher

**Description:** Searches for specific keywords within text-based files

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/keyword_searcher
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/keyword_searcher
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### keyword_search

**Description:** Searches file contents for user-specified keywords

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/keyword_search
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/keyword_search
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### git_status

**Description:** Shows the git status of each file

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/git_status
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/git_status
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### dirs

**Description:** Shows directories metadata

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/dirs
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/dirs
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

### file_meta

**Description:** Displays the file metadata of each file

**Version:** 0.2.0

**Installation Options:**

1. Using LLA install command:
```bash
lla install --dir path/to/lla/plugins/file_meta
```

2. Manual installation:
```bash
git clone https://github.com/triyanox/lla
cd lla/plugins/file_meta
cargo build --release
```

Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.

