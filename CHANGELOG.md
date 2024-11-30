# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

### Enhanced

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
