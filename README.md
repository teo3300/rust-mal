# rust-mal

Trying and implementing [mal](https://github.com/kanaka/mal) project from [kanaka](https://github.com/kanaka) in Rust

## Why

Because I need to improve my knowledge of rust and I also don't know how to use lisp

## Installation

Running `make install` should suffice

### Installation Configuration
Parameters can be set for installation via environment varialbes:
- `MAL_HOME`
  - Directory containing the core, libraries and config
  - defaults to `~/.config/mal`
- `BINARY_DIR`
  - Destination to install the binary, must be included in `PATH` to work properly
  - defaults to `/usr/local/bin`