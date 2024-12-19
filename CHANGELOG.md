# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.6] - 2024-12-18

### Added

- Shell completion support for various shells like `bash`, `zsh`, `fish`, and `elvish`
- Interactive fuzzy file search functionality with the `--fuzzy` flag
- New `FuzzyFormatter [unstable]` and `FuzzyLister [unstable]` components for fuzzy search support, this will be improved in future releases to handle very large directories

### Changed

- Updated `Args` struct with new completion command and fuzzy_format option, now you can generate completions for your own shell using `lla completion <shell> <path[optional]>`
- Enhanced command handling to support shell completion functionality
- Refactored file listing and formatting logic to support fuzzy search
- Modified dependencies to include `clap_complete`

## [0.3.5] - 2024-12-16

### Added

- A theming system to customize the look of `lla`
- New configuration option `theme`
- An extensive theming preset library
- Add the `--no-color` flag to disable color output, and works will all listing formats
- New package managers support
- Include window builds in the releases

### Fixed

- Minor fixes and improvements
- Stability improvements

## [0.3.4] - 2024-12-14

### Added

- The ability to set plugins path with `config --set`

## [0.3.3] - 2024-12-14

### Added

- New configuration options like `sort`, `filter`, `icons`

### Changed

- Better error handling
- Better and much cleaner plugins installation process
- Revised config settings
- Refactor the main entry point
- Enhanced plugin update mechanism

### Fixed

- Fixed the layout and style of the plugin installation process
- Fixed plugins loading

## [0.3.2] - 2024-12-14

### Added

- New configuration options like `sort`, `filter`, `icons`

### Changed

- Better error handling
- Better and much cleaner plugins installation process
- Revised config settings
- Refactor the main entry point
- Enhanced plugin update mechanism

### Fixed

- Fixed the layout and style of the plugin installation process

## [0.3.1] - 2024-12-12

### Added

- Plugin system redesign:
  - Protocol Buffers message passing architecture
  - C API compatibility
  - ABI version verification
  - Improved documentation
  - Enhanced plugin management interface
  - Plugin cleanup command (`lla clean`)
  - Improved plugin discovery and loading
  - Plugin update functionality improved
  - Improved the functionality and look of all plugins
- Command shortcuts system:
  - Store and manage plugin commands as shortcuts
  - CLI commands for shortcut management (`lla shortcut add/remove/list`)
  - Configuration file storage with descriptions
  - Support for custom arguments
  - Simplified command syntax
- Sorting improvements:
  - Reverse sorting (`-r`, `--sort-reverse`)
  - Directory-first option (`--sort-dirs-first`)
  - Case-sensitive sorting (`--sort-case-sensitive`)
  - Natural number sorting (`--sort-natural`)
- Filter system updates:
  - Multiple pattern support with comma separation
  - AND operations using `+` prefix
  - Logical operations (AND, OR, NOT, XOR)
  - Glob pattern matching
  - Case sensitivity options
- Additional features:
  - Icon support across formats
  - Updated `sizemap`, `timeline` and `git` views
  - Selective plugin installation
  - Command shortcut system

### Changed

- Performance optimizations for sorting
- Improved filter matching
- Plugin system reliability updates
- Refined sizemap visualization
- Updated plugin interfaces
- Interface improvements
- General stability enhancements

### Fixed

- Pregenerate protobuf bindings
- Plugin ABI compatibility
- Case-sensitive search behavior
- Directory sorting issues
- Numeric filename sorting

## [0.3.0] - 2024-12-11

### Added

- Plugin system redesign:
  - Protocol Buffers message passing architecture
  - C API compatibility
  - ABI version verification
  - Improved documentation
  - Enhanced plugin management interface
  - Plugin cleanup command (`lla clean`)
  - Improved plugin discovery and loading
  - Plugin update functionality improved
  - Improved the functionality and look of all plugins
- Command shortcuts system:
  - Store and manage plugin commands as shortcuts
  - CLI commands for shortcut management (`lla shortcut add/remove/list`)
  - Configuration file storage with descriptions
  - Support for custom arguments
  - Simplified command syntax
- Sorting improvements:
  - Reverse sorting (`-r`, `--sort-reverse`)
  - Directory-first option (`--sort-dirs-first`)
  - Case-sensitive sorting (`--sort-case-sensitive`)
  - Natural number sorting (`--sort-natural`)
- Filter system updates:
  - Multiple pattern support with comma separation
  - AND operations using `+` prefix
  - Logical operations (AND, OR, NOT, XOR)
  - Glob pattern matching
  - Case sensitivity options
- Additional features:
  - Icon support across formats
  - Updated `sizemap`, `timeline` and `git` views
  - Selective plugin installation
  - Command shortcut system

### Changed

- Performance optimizations for sorting
- Improved filter matching
- Plugin system reliability updates
- Refined sizemap visualization
- Updated plugin interfaces
- Interface improvements
- General stability enhancements

### Fixed

- Plugin ABI compatibility
- Case-sensitive search behavior
- Directory sorting issues
- Numeric filename sorting

## [0.2.10] - 2024-11-30

### Added

- New display formats for enhanced visualization:
  - `git`: Display Git status information for files
  - `grid`: Present files in an organized grid layout
  - `sizemap`: Visualize file sizes with proportional representation
  - `table`: Show files in a structured table format
  - `timeline`: Group files by creation/modification dates
- Interactive plugin management system
- Plugin update functionality via CLI
- Extended configuration options for customization
- Plugin support for default and long format customization

### Changed

- Significant performance improvements:
  - Optimized tree view rendering
  - More efficient recursive file listing
  - Better memory management for large directories
- Plugin system improvements:
  - Refined plugin interface for better integration
  - More robust plugin installation process
  - Enhanced plugin discovery and loading
- Sorting functionality:
  - More accurate file sorting across all formats
  - Improved performance for large directory sorting

### Fixed

- Memory leaks in recursive directory listing
- Plugin installation reliability issues
- Color output consistency across different formats

## [0.2.9] - 2024-11-27

### Changed

- Plugin interface versioning

### Fixed

- Plugin interface versioning
- GitHub Actions workflows

## [0.2.8] - 2024-01-09

### Added

- Multi-architecture support for all major platforms
- Cargo workspace setup for better dependency management
- GitHub Actions workflows for automated releases
- SHA256 checksums for all binary artifacts

### Changed

- Migrated to Cargo workspace structure
- Updated build system to use workspace inheritance
- Improved cross-compilation support

### Fixed

- Build consistency across different platforms
- Plugin interface versioning
