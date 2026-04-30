# Building & Deploying

## Build

```bash
cargo build --release
```

The binary appears at `target/release/obsidian-mcp` (or `.exe` on Windows). Current release size: **3.8 MB**.

## Release Profile

The `Cargo.toml` release profile is tuned for minimal binary size:

```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = true            # Link-time optimization
strip = true          # Strip debug symbols
panic = "abort"       # Smaller unwinding code
codegen-units = 1     # Better optimization at cost of compile time
```

## Cross-Compilation

```bash
# Linux (x86_64)
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# macOS (Apple Silicon)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Windows
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Cross-compilation requires the appropriate toolchain (linker, sysroot). On Linux, install the `musl` target for a fully static binary:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

## Dependency Audit

```bash
cargo audit    # Check for known vulnerabilities
cargo deny check  # License and security policy checks
```

## Deployment Checklist

- [ ] Binary built in release mode
- [ ] `--test-connection` passes
- [ ] Binary on PATH or referenced by full path in client config
- [ ] `.env` file in place with correct API key
- [ ] Obsidian running with Local REST API plugin enabled
