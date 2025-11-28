# Development guide

## Prerequisites
- Rust 1.75+ (edition 2024)
- Protobuf compiler (`protoc`)
- Docker (optional, for testing workflows with `act`)
- Just (optional but recommended, for simplified development tasks)

### Installing Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y protobuf-compiler
```

**macOS:**
```bash
brew install protobuf
```

**Windows:**
```bash
choco install protoc
```

### Installing Just (Optional but Recommended)
`just` is a command runner that simplifies common development tasks:

**Ubuntu/Debian/Windows:**
```bash
cargo install just
```

**macOS:**
```bash
brew install just
```

## Quick Start with Just

If you have `just` installed, you can use these convenient commands:

```bash
# List all available commands
just

# Start development environment (server + UI with hot reload)
just dev

# Run server only with auto-reload
just dev-server

# Run UI only with hot reload
just dev-ui

# Build UI assets
just build-ui

# Build release binary (includes UI build)
just build

# Run all tests
just test

# Run linters and format checks
just check

# Clean build artifacts
just clean
```

## Building

```bash
# Development build (uses system DuckDB library)
cargo build

# Release build with bundled DuckDB (portable, no system dependencies)
cargo build --release --features bundled

# Build specific package
cargo build -p glint-cli --release --features bundled

# Development build uses system DuckDB for faster compilation
# Release build uses bundled DuckDB for portability
```

### DuckDB configuration

Glint uses different DuckDB linking strategies for development and release:

**Development (default):**
- Uses system-installed DuckDB library
- Much faster compilation (no need to build libduckdb-sys)
- Requires DuckDB installed on your system
- On NixOS: provided by `flake.nix`

**Release (`--features bundled`):**
- Bundles DuckDB statically
- Fully portable binary (no system dependencies)
- Longer compilation time
- Required for distribution packages

```bash
# Install DuckDB system library
# Ubuntu/Debian
sudo apt-get install libduckdb-dev

# macOS
brew install duckdb

# Arch Linux
sudo pacman -S duckdb

# NixOS
nix develop  # Uses flake.nix
```

## Running

```bash
# Run with cargo
cargo run -p glint-cli -- serve

# Run built binary
./target/release/glint serve

# Or use just for development with hot reload
just dev
```

## Testing

```bash
cargo test --workspace

# Run specific package tests
cargo test -p glint           # Core library tests (59 tests)
cargo test -p glint-collector # Collector tests (47 tests)
cargo test -p glint-server    # Server tests (18 tests)

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_export_traces_http

# Or use just
just test
```

## Code quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-targets --all-features

# Fix clippy warnings
cargo clippy --workspace --all-targets --all-features --fix

# Or use just to run all checks
just check
```

### Creating a release

1. Update version in `Cargo.toml`:
```toml
[workspace.package]
version = "0.1.3"
```

2. Update CHANGELOG.md

3. Commit and tag:
```bash
git add -A
git commit -m "Release v0.1.3"
git tag v0.3.0
git push origin main --tags
```

4. GitHub Actions will automatically:
   - Build all binaries
   - Create packages
   - Create GitHub release

## Package building

### .deb Package (Debian/Ubuntu)

```bash
# Install cargo-deb
cargo install cargo-deb

# Build package
cargo deb -p glint-cli

# Install locally
sudo dpkg -i target/debian/glint_*.deb

# Test installation
glint --version
```

### .rpm Package (RHEL/Fedora)

```bash
# Install cargo-generate-rpm
cargo install cargo-generate-rpm

# Build binary first
cargo build --release --bin glint

# Generate RPM
cargo generate-rpm -p glint-cli

# Install locally
sudo rpm -i target/generate-rpm/glint-*.rpm

# Test installation
glint --version
```

## Database management

Glint uses smart project-based database management:

```bash
# Show database info
glint info

# Clean current project database
glint clean

# Clean all databases
glint clean --all
```

Databases are stored in:
- Linux/macOS: `~/.config/glint/<project>.db`
- Windows: `%APPDATA%/glint/<project>.db`

## Adding New Features

### Adding a New CLI Command

1. Add command to `glint-cli/src/main.rs`:
```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    MyCommand {
        #[arg(long)]
        my_option: Option<String>,
    },
}
```

2. Handle command in `main()`:
```rust
Commands::MyCommand { my_option } => {
    // Implementation
}
```

### Adding a New API Endpoint

1. Add route in `glint-server/src/routes.rs`:
```rust
pub async fn my_endpoint(
    State(state): State<AppState>,
    Query(params): Query<MyParams>,
) -> Result<Json<MyResponse>, StatusCode> {
    // Implementation
}
```

2. Register route in `create_router()`:
```rust
.route("/api/my-endpoint", get(my_endpoint))
```

3. Add tests:
```rust
#[tokio::test]
async fn test_my_endpoint() {
    // Test implementation
}
```

### Adding a New Storage Query

1. Add method in `glint/src/storage/mod.rs`:
```rust
pub fn my_query(&self, params: MyParams) -> Result<Vec<MyType>> {
    let conn = self.conn.lock().unwrap();
    // DuckDB query
}
```

2. Add tests:
```rust
#[test]
fn test_my_query() {
    let storage = Storage::new_in_memory().unwrap();
    // Test implementation
}
```

## Troubleshooting

### DuckDB Locking Issues

If you get "database is locked" errors:
- Stop all running `glint serve` instances
- The server holds a write lock on the database
- Use `glint info` to check database location

### Protobuf Compilation Errors

```bash
# Make sure protoc is installed
protoc --version

# Clean and rebuild
cargo clean
cargo build
```

### Test Failures

```bash
# Run with verbose output
cargo test --workspace -- --nocapture

# Run specific failing test
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --workspace` or `just test`
5. Run linters: `cargo fmt --all && cargo clippy --workspace` or `just check`
6. Submit a pull request

## Performance Tips

### Development Builds

Use `sccache` for faster compilation:
```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Release Builds

For maximum performance:
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Available Just Commands

For your reference, here are all the commands defined in the `justfile`:

| Command | Description |
|---------|-------------|
| `just` | List all available commands |
| `just dev` | Start both server and UI with hot reload |
| `just dev-server` | Run server with auto-reload (cargo-watch) |
| `just dev-ui` | Run UI development server with hot reload |
| `just build-ui` | Build UI production assets |
| `just build` | Build release binary (includes UI build) |
| `just test` | Run all workspace tests |
| `just check` | Run clippy and format checks |
| `just clean` | Clean all build artifacts |

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)
- [DuckDB Documentation](https://duckdb.org/docs/)
- [Axum Documentation](https://docs.rs/axum/)
- [Just Documentation](https://just.systems/)
