# Development Environment

## Supported Baseline

The current development baseline is Debian 13 with the Rust toolchain supplied by Debian packages.

Verified on 2026-09-14:

- `rustc 1.85.1`
- `cargo 1.85.1`
- `rustfmt 1.8.0`
- `clippy 0.1.85`

## Debian Installation

Install the Rust compiler, Cargo, formatter, and Clippy with:

```bash
sudo apt-get update
sudo apt-get install -y rustc cargo rustfmt rust-clippy
```

The package name for Clippy on Debian is `rust-clippy`, not `clippy`.

This installs the verified stable Debian baseline (`rustc 1.85.1`). The
repository also requires the Rust standard-library sources for rust-analyzer:

```bash
sudo apt-get install -y rust-src rust-analyzer
```

For a newer rust-analyzer and compiler on Debian 13, use the configured
backports repository and keep the Rust packages at the same version:

```bash
sudo apt-get update
sudo apt-get -t trixie-backports install -y rustc cargo rust-src rust-analyzer rustfmt rust-clippy
```

Do not mix a backports `rust-analyzer` with an older compiler and standard
library source. The compiler, `rust-src`, and rust-analyzer packages should
come from the same Debian suite.

Verify the installation:

```bash
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
```

## Workspace Validation

Run these commands from the repository root:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The current Phase 2 workspace does not require Docker or Docker Compose. Containers can be introduced later if platform-specific or reproducible cross-target builds require them; the native Debian toolchain is currently the simplest verified environment.

## Toolchain Policy

The repository includes `rust-toolchain.toml` and targets the stable Rust
channel with `rust-src`, rustfmt, and Clippy components. rustup users receive
these components automatically. APT users must install the matching Debian
packages manually because `rust-toolchain.toml` is not interpreted by the
system-provided compiler.

Verify rust-analyzer:

```bash
rust-analyzer --version
rustc --version
```

The exact compiler version may vary between supported distributions; CI should
eventually pin and verify the intended compiler version before release builds.
