# Sursface

Sursface is a cross-platform rendering library designed to simplify the development of high-performance graphical applications. It provides support for multiple platforms, automatically generating a window and hooking it up with wgpu, which can be directly used through sursface's crate.

## Features

- **Cross-Platform Support**: Compile and run on various platforms including Linux, Windows, and MacOS (untested).
- **WASM Support**: Automatically generate JavaScript bindings for the `wasm32-unknown` Nix target, enabling web-based applications.

## Getting Started

### Prerequisites

To compile Sursface, ensure you have the following tools installed: Rust, Cargo, [Nix](https://nixos.org/download.html) (optional but recommended).

### Installation

Clone the repository:

```sh
git clone https://github.com/yourusername/sursface.git
cd sursface
```

### Building with Cargo

You can compile Sursface on your native platform using Cargo:

```sh
cargo build --release
```

### Building with Nix

To build a binary inside the Nix store ready to run locally:

```sh
nix build .
```

### Cross-Compiling with Nix

Sursface supports cross-compilation to various targets. Here are some examples:

#### Compiling for Linux

```sh
nix build .#x86_64-linux
```

#### Compiling for Windows

```sh
nix build .#x86_64-windows
```

#### Compiling for WebAssembly (WASM)

```sh
nix build .#wasm32-unknown
```

## Running Examples

There are example applications included in the `examples` directory. To run an example:

```sh
cargo run hello_window
```