# Bl1nk Agents Manager - Development Commands

# --- Default ---
default:
    @just --list

# --- Build Commands ---

# Standard build (lightweight, no bundled features)
build:
    @echo "Building standard release binary..."
    cargo build --release

# Build with bundled PMAT (recommended)
build-bundled:
    @echo "Building release binary with bundled PMAT..."
    cargo build --release --features bundle-pmat

# Build with bundled PMAT and all language features
build-full:
    @echo "Building release binary with bundled PMAT (full language support)..."
    cargo build --release --features bundle-pmat-full

# --- Run Commands ---

# Run the standard build
run: build
    @echo "Running standard release binary..."
    ./target/release/bl1nk-agents-manager

# Run the bundled build
run-bundled: build-bundled
    @echo "Running release binary with bundled PMAT..."
    ./target/release/bl1nk-agents-manager

# --- Development & Quality ---

# Run in development mode with hot-reload
dev:
    @echo "Starting development mode with hot-reload..."
    cargo watch -x 'run --features bundle-pmat'

# Run all tests
test:
    @echo "Running all tests..."
    cargo test --all-features

# Check for compilation errors quickly
check:
    @echo "Running cargo check..."
    cargo check --all-features

# Format code
fmt:
    @echo "Formatting code..."
    cargo fmt --all

# Run Clippy linter with strict warnings
clippy:
    @echo "Running clippy linter..."
    cargo clippy --all-features -- -D warnings

# Run full workflow checks (format, clippy, check, agents)
verify:
    @echo "Running full project workflow..."
    python3 scripts/project_workflow.py all

# Update auto-generated project map
map:
    @echo "Updating project map..."
    python3 scripts/update_project_map.py

# Audit skill registry against skills/ folders
audit-skills:
    @echo "Auditing skills registry..."
    python3 scripts/project_workflow.py audit-skills

# Validate agents metadata and files
validate-agents:
    @python3 scripts/validate_agents.py

# Fix agent metadata and formatting
fix-agents:
    @python3 scripts/fix_agents.py

# --- Installation & Cleanup ---

# Install the standard binary to ~/.local/bin
install: build
    @echo "Installing standard binary to ~/.local/bin..."
    mkdir -p ~/.local/bin
    cp target/release/bl1nk-agents-manager ~/.local/bin/
    @echo "Installed to ~/.local/bin/bl1nk-agents-manager"

# Install the bundled binary
install-bundled: build-bundled
    @echo "Installing bundled PMAT binary to ~/.local/bin..."
    mkdir -p ~/.local/bin
    cp target/release/bl1nk-agents-manager ~/.local/bin/
    @echo "Installed to ~/.local/bin/bl1nk-agents-manager"

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    cargo clean

# --- Documentation & Setup ---

# Generate and open documentation
doc:
    @echo "Generating documentation..."
    cargo doc --no-deps --open

# Setup development environment
setup:
    @echo "Installing development tools..."
    rustup component add rustfmt clippy
    cargo install cargo-watch
    @echo "Development tools installed successfully!"
