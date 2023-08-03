# `lla` - The Elegant `ls` Alternative

## Overview

`lla` (Lightweight ls Alternative) is an elegant and efficient command-line utility written in Rust, designed to be a fast and minimalistic replacement for the classic "ls" command. Say goodbye to cluttered outputs and embrace a refreshing listing experience with `lla`. Whether you're dealing with small directories or vast file landscapes, `lla` delivers lightning-fast performance with its minimalist design.

<img src="lla.png" alt="lla" width="100%">

## Features

- **Simplicity & Speed**: `lla` is crafted to be lightweight and lightning-fast, offering a sleek, no-nonsense listing experience.

- **Long Format**: Get detailed file information with the `-l` flag, displaying essential details for each file.

- **Sorting Options**: Sort files by name, size, or date with ease using the `-s` flag.

- **Filtering Flexibility**: Filter files by extension or pattern using the `-f` flag, with support for dot (.) prefix for extensions.

- **Recursive Exploration**: Effortlessly explore subdirectories with recursive listing, enabled by the `-r` flag.

- **Custom Depth**: Control recursion with precision by setting the maximum depth using the `-d` option.

## Installation

`lla` can be easily installed using Cargo, the package manager for Rust.

1. **Install Rust**: Make sure you have Rust installed on your system. If not, [install Rust](https://www.rust-lang.org/learn/get-started).

2. **Install `lla`**: Open your terminal and run the following command to install `lla` using Cargo:

```bash
cargo install lla
```

3. **Ready to Use**: Once the installation is complete, you can start using `lla` immediately. Simply type ``lla`` in your terminal, and let the magic begin!

### NetBSD

On NetBSD, a pre-compiled binary is available from the official repositories. To install it, simply run:
```bash
pkgin install lla
```

## Usage

```
lla [FLAGS] [OPTIONS] [DIRECTORY]
```

### Flags

- `-l`, `--long`: Utilize the long listing format, providing detailed information for each file.

- `-r`, `--recursive`: Explore subdirectories recursively.

### Options

- `-s`, `--sort [SORT_BY]`: Sort files by name, size, or date. Values for SORT_BY: "name", "size", or "date".

- `-f`, `--filter [FILTER_BY]`: Filter files by extension or pattern. Dot (.) prefix for extension filtering.

- `-d`, `--depth [DEPTH]`: Set the maximum depth of recursion when using recursive mode.

### Arguments

- `DIRECTORY`: The directory to list. Default is the current directory.

## Examples

- List files and directories in the current directory:

```bash
lla
```

- List files and directories in a specific directory:

```bash
lla /path/to/directory
```

- Use the long listing format:

```bash
lla -l
```

- Sort files by size:

```bash
lla -s size
```

- Filter files by extension:

```bash
lla -f .txt
```

- Explore subdirectories recursively with a depth limit:

```bash
lla -r -d 3
```

- Combine flags and options:

```bash
lla -l -r -s size -f .txt -d 3
# or
lla -lrs size -f .txt -d 3
```

## Contributing

`lla` is an open-source project that warmly welcomes contributions from the community. If you encounter any issues or have exciting ideas for improvements, don't hesitate to submit a pull request or open an issue.

## License

This project is licensed under the [MIT License](LICENSE), allowing you to use, modify, and distribute it freely. Embrace the elegance of `lla`, the perfect replacement for the traditional "ls" command. 
