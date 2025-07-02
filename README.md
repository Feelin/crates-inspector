# crates-inspector 🔍

A command-line tool for managing Rust package dependencies directly in your terminal.

[![crates.io](https://img.shields.io/crates/v/crates-inspector)](https://crates.io/crates/crates-inspector)
![crates.io](https://img.shields.io/crates/d/crates-inspector)
[![dependency status](https://deps.rs/repo/github/Feelin/crates-inspector/status.svg)](https://deps.rs/repo/github/Feelin/crates-inspector)
[![Minimum Stable Rust Version](https://img.shields.io/badge/Rust-1.81.0-blue?color=fc8d62&logo=rust)](https://github.com/rust-lang/rust/blob/master/RELEASES.md#version-1810-2024-09-05)
[![License: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](https://spdx.org/licenses/GPL-3.0-only.html)



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

You need the Rust toolchain installed on your system to build `lstr`.

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

## Examples

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
![image](https://camo.githubusercontent.com/cc71d76b2a1573f27c8f6a4e4008467e93c393003e6d07951f76891197ad2577/68747470733a2f2f6769746875622d70726f64756374696f6e2d757365722d61737365742d3632313064662e73332e616d617a6f6e6177732e636f6d2f373339313737332f3436313332333334302d37343532643636622d613063642d343935662d613133612d3731653437316664663634362e706e673f582d416d7a2d416c676f726974686d3d415753342d484d41432d53484132353626582d416d7a2d43726564656e7469616c3d414b494156434f44594c5341353350514b345a41253246323032353037303225324675732d656173742d312532467333253246617773345f7265717565737426582d416d7a2d446174653d3230323530373032543032343232335a26582d416d7a2d457870697265733d33303026582d416d7a2d5369676e61747572653d3462326366653630333139393864343263353164346132633436306564386432623065343362363230363136623963313438313731363666376337373632376126582d416d7a2d5369676e6564486561646572733d686f7374)




## Support Me
If you like this project, you can support me in the following ways:

- [⭐️ Give this project a Star](https://github.com/Feelin/crates-inspector)
- Share with your friends


