# crates-inspector 🔍

A command-line tool for managing Rust package dependencies directly in your terminal.

[![crates.io](https://img.shields.io/crates/v/crates-inspector)](https://crates.io/crates/crates-inspector)
![crates.io](https://img.shields.io/crates/d/crates-inspector)
[![dependency status](https://deps.rs/repo/github/Feelin/crates-inspector/status.svg)](https://deps.rs/repo/github/Feelin/crates-inspector)
[![Minimum Stable Rust Version](https://img.shields.io/badge/Rust-1.81.0-blue?color=fc8d62&logo=rust)](https://github.com/rust-lang/rust/blob/master/RELEASES.md#version-1810-2024-09-05)
[![License: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](https://spdx.org/licenses/GPL-3.0-only.html)[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FFeelin%2Fcrates-inspector.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FFeelin%2Fcrates-inspector?ref=badge_shield)




## Features ✨

- **Dependency Listing**:
  - List all dependencies in your project
  - Show only direct dependencies
- **Sorting**:
  - Order by package name (alphabetical)
  - Order by dependency size
  - Order by dependency version
- **Filtering**:
  - Filter dependencies by name 
- **Project Statistics**:
  - Count total dependencies
  - Calculate combined size of dependencies
- **Lightweight**:
  - Fast, native Rust implementation

## Installation 📦

### From crates.io:
```bash
cargo install crates-inspector
```
### From source (all platforms)

You need the Rust toolchain installed on your system to build `crates-inspector`.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Feelin/crates-inspector.git
    cd crates-inspector
    ```
2.  **Build and install using Cargo:**
    ```bash
    cargo install --path .
    ```
  
## Usage

### Examples

**1. List the dependencies of the current directory**

```bash
crates-inspector
```
**2. List the contents of the other directory**

```bash
crates-inspector -p "$RUST_PROJECT_PATH"
```

### Keyboard controls

| Key(s)  | Action                                                                                                                                      |
| :------ | :------------------------------------------------------------------------------------------------------------------------------------------ |
| `↑` / `k` | Move selection up. |
| `↓` / `j` | Move selection down. |
| `→` / `l` | Select child package. |
| `←` / `h` | Select parent package. |
| `Enter` | Open documentation in browser. |
| `q` / `Esc` | Quit the application normally. |
| `a` | Show all dependencies. |
| `d` | Show direct dependencies. |
| `f` | Filter by package name. |
| `s` | Sort or reverse. |


## Screenshot 📸
<img width="984" alt="image" src="https://github.com/user-attachments/assets/5a075624-cd79-4d96-9489-ef0ad6d36959" />





## Support Me
If you like this project, you can support me in the following ways:

- [⭐️ Give this project a Star](https://github.com/Feelin/crates-inspector)
- Share with your friends




## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FFeelin%2Fcrates-inspector.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FFeelin%2Fcrates-inspector?ref=badge_large)