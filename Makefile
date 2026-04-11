# ==============================================================================
# Makefile for Gemini MCP Proxy
# ==============================================================================

# .PHONY tells make that these are not files, so it should always run the command.
.PHONY: help build build-bundled build-full run dev test check fmt clippy clean install doc setup lint spellcheck all-check

# --- Default Behavior ---
# `make` or `make all` will run the default build.
all: build

# --- Help & Documentation ---
help:
	@echo "Gemini MCP Proxy - Development Commands"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@echo "  help                  - Show this help message."
	@echo "  setup                 - Install required development tools (rustfmt, clippy, cargo-watch)."
	@echo ""
	@echo "Build Targets:"
	@echo "  build                 - Build standard release binary (lightweight)."
	@echo "  build-bundled         - Build release binary with bundled PMAT (recommended)."
	@echo "  build-full            - Build release binary with bundled PMAT + all language support."
	@echo ""
	@echo "Development & Testing:"
	@echo "  run                   - Run the standard release binary."
	@echo "  run-bundled           - Run the release binary with bundled PMAT."
	@echo "  dev                   - Run in dev mode with hot-reload (watches for file changes)."
	@echo "  test                  - Run all tests, including feature-specific ones."
	@echo "  check                 - Run cargo check for quick compilation checks."
	@echo "  fmt                   - Format all code according to project style."
	@echo "  clippy                - Run clippy linter to find potential issues (with strict warnings)."
	@echo "  lint                  - Run all linting tools (fmt + check + clippy)."
	@echo "  spellcheck            - Run codespell to check for spelling errors."
	@echo "  all-check             - Run all checks (lint + spellcheck + test)."
	@echo ""
	@echo "Deployment & Cleanup:"
	@echo "  install               - Install the standard release binary to ~/.local/bin."
	@echo "  install-bundled       - Install the bundled PMAT release binary to ~/.local/bin."
	@echo "  clean                 - Clean all build artifacts."
	@echo "  doc                   - Generate and open project documentation."


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
	./target/release/gemini-mcp-proxy

# Run the bundled build
run-bundled: build-bundled
	@echo "Running release binary with bundled PMAT..."
	./target/release/gemini-mcp-proxy

# Run in development mode with hot-reload
dev:
	@echo "Starting development mode with hot-reload..."
	cargo watch -x 'run --features bundle-pmat'


# --- Testing and Quality ---

# Run all tests, including those that require features
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

# Run all linting tools (fmt + clippy + check)
lint:
	@echo "Running all linters..."
	@echo "--- Running cargo fmt ---"
	cargo fmt --all --check
	@echo "--- Running cargo check ---"
	cargo check --all-features
	@echo "--- Running cargo clippy ---"
	cargo clippy --all-features -- -D warnings
	@echo "Lint check complete!"

# Spell check using codespell
spellcheck:
	@echo "Running spell check..."
	@command -v codespell >/dev/null 2>&1 && codespell --config .codespellrc . || echo "codespell not installed, skipping..."

# Run all checks (lint + spellcheck + test)
all-check: lint spellcheck test
	@echo "All checks passed!"


# --- Installation and Cleanup ---

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Install the standard binary
install: build
	@echo "Installing standard binary to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp target/release/gemini-mcp-proxy ~/.local/bin/
	@echo "Installed to ~/.local/bin/gemini-mcp-proxy"
	@echo "Make sure ~/.local/bin is in your PATH"

# Install the bundled binary
install-bundled: build-bundled
	@echo "Installing bundled PMAT binary to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp target/release/gemini-mcp-proxy ~/.local/bin/
	@echo "Installed to ~/.local/bin/gemini-mcp-proxy"
	@echo "Make sure ~/.local/bin is in your PATH"


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
