# ==============================================================================
# Makefile for BL1NK Agents Manager
# ==============================================================================

<<<<<<< HEAD
.PHONY: help build build-bundled build-full run dev test check fmt clippy clean install doc setup lint spellcheck all-check \
        parallel parallel-verbose fmt-only clippy-only test-only review review-comment \
        commitlint commitlint-range changelog \
        bump-patch bump-minor bump-major security-check security-update
=======
# .PHONY tells make that these are not files, so it should always run the command.
.PHONY: help build build-bundled build-full run dev test check fmt clippy clean install doc setup lint spellcheck all-check
>>>>>>> origin/dev

# --- Default Behavior ---
all: build

# --- Help & Documentation ---
help:
	@echo "BL1NK Agents Manager - Development Commands"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Build Targets:"
	@echo "  build                 - Build standard release binary"
	@echo "  build-bundled         - Build with bundled PMAT (recommended)"
	@echo "  build-full            - Build with bundled PMAT + all features"
	@echo ""
<<<<<<< HEAD
	@echo "Development:"
	@echo "  run                   - Run standard release binary"
	@echo "  run-bundled           - Run bundled PMAT binary"
	@echo "  dev                   - Hot-reload development mode"
=======
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
>>>>>>> origin/dev
	@echo ""
	@echo "Quality Checks:"
	@echo "  test                  - Run all tests"
	@echo "  check                 - Quick compilation check"
	@echo "  fmt                   - Format code"
	@echo "  clippy                - Run clippy linter"
	@echo "  lint                  - Run all linters (fmt + clippy + check)"
	@echo "  spellcheck            - Check spelling"
	@echo "  all-check             - Run all checks sequentially"
	@echo "  parallel              - Run fmt + clippy + test in PARALLEL (fast)"
	@echo "  parallel-verbose      - Run parallel with verbose output"
	@echo "  review [TARGET]       - Code review (local changes, PR#, or file)"
	@echo "  review-comment PR#    - Code review + post inline comments"
	@echo ""
	@echo "Release Management:"
	@echo "  bump-patch            - Bump patch version (0.1.0 -> 0.1.1)"
	@echo "  bump-minor            - Bump minor version (0.1.0 -> 0.2.0)"
	@echo "  bump-major            - Bump major version (0.1.0 -> 1.0.0)"
	@echo "  changelog             - Generate CHANGELOG.md from commits"
	@echo "  commitlint            - Lint last commit message"
	@echo "  commitlint-range      - Lint last 10 commits"
	@echo ""
	@echo "Security:"
	@echo "  security-check        - Check dependencies for vulnerabilities"
	@echo "  security-update       - Update dependencies and check security"
	@echo ""
	@echo "Deployment:"
	@echo "  install               - Install to ~/.local/bin"
	@echo "  install-bundled       - Install bundled binary"
	@echo "  clean                 - Clean build artifacts"
	@echo ""
	@echo "Documentation:"
	@echo "  doc                   - Generate documentation"
	@echo "  setup                 - Install dev tools"


# ==============================================================================
# Build Commands
# ==============================================================================

build:
	@echo "Building standard release binary..."
	cargo build --release

build-bundled:
	@echo "Building release binary with bundled PMAT..."
	cargo build --release --features bundle-pmat

build-full:
	@echo "Building release binary with bundled PMAT (full language support)..."
	cargo build --release --features bundle-pmat-full


# ==============================================================================
# Run Commands
# ==============================================================================

run: build
	@echo "Running standard release binary..."
	./target/release/bl1nk-agents-manager

run-bundled: build-bundled
	@echo "Running release binary with bundled PMAT..."
	./target/release/bl1nk-agents-manager

dev:
	@echo "Starting development mode with hot-reload..."
	cargo watch -x 'run --features bundle-pmat'


# ==============================================================================
# Testing and Quality (Sequential)
# ==============================================================================

test:
	@echo "Running all tests..."
	cargo test --all-features

check:
	@echo "Running cargo check..."
	cargo check --all-features

fmt:
	@echo "Formatting code..."
	cargo fmt --all

clippy:
	@echo "Running clippy linter..."
	cargo clippy --all-features -- -D warnings

<<<<<<< HEAD
=======
# Run all linting tools (fmt + clippy + check)
>>>>>>> origin/dev
lint:
	@echo "Running all linters..."
	@echo "--- Running cargo fmt ---"
	cargo fmt --all --check
	@echo "--- Running cargo check ---"
	cargo check --all-features
	@echo "--- Running cargo clippy ---"
	cargo clippy --all-features -- -D warnings
	@echo "Lint check complete!"
<<<<<<< HEAD
=======

# Spell check using codespell
spellcheck:
	@echo "Running spell check..."
	@command -v codespell >/dev/null 2>&1 && codespell --config .codespellrc . || echo "codespell not installed, skipping..."

# Run all checks (lint + spellcheck + test)
all-check: lint spellcheck test
	@echo "All checks passed!"

>>>>>>> origin/dev

spellcheck:
	@echo "Running spell check..."
	@command -v codespell >/dev/null 2>&1 && codespell --config .codespellrc . || echo "codespell not installed, skipping..."

all-check: lint spellcheck test
	@echo "All checks passed!"


# ==============================================================================
# Parallel Checks (Fast)
# ==============================================================================

parallel:
	@bash scripts/parallel-check.sh

parallel-verbose:
	@bash scripts/parallel-check.sh --verbose

fmt-only:
	@bash scripts/parallel-check.sh --fmt

clippy-only:
	@bash scripts/parallel-check.sh --clippy

test-only:
	@bash scripts/parallel-check.sh --test

review:
	@bash scripts/review.sh $(filter-out $@,$(MAKECMDGOALS))

review-comment:
	@bash scripts/review.sh $(filter-out $@,$(MAKECMDGOALS)) --comment


# ==============================================================================
# Release Management
# ==============================================================================

bump-patch:
	@bash scripts/bumpversion.sh patch

bump-minor:
	@bash scripts/bumpversion.sh minor

bump-major:
	@bash scripts/bumpversion.sh major

changelog:
	@bash scripts/generate-changelog.sh

commitlint:
	@bash scripts/commitlint.sh --last

commitlint-range:
	@bash scripts/commitlint.sh --range HEAD~10..HEAD


# ==============================================================================
# Security
# ==============================================================================

security-check:
	@bash scripts/update-security.sh --check-only

security-update:
	@bash scripts/update-security.sh


# ==============================================================================
# Installation and Cleanup
# ==============================================================================

clean:
	@echo "Cleaning build artifacts..."
	cargo clean

install: build
	@echo "Installing standard binary to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp target/release/bl1nk-agents-manager ~/.local/bin/
	@echo "Installed to ~/.local/bin/bl1nk-agents-manager"
	@echo "Make sure ~/.local/bin is in your PATH"

install-bundled: build-bundled
	@echo "Installing bundled PMAT binary to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp target/release/bl1nk-agents-manager ~/.local/bin/
	@echo "Installed to ~/.local/bin/bl1nk-agents-manager"
	@echo "Make sure ~/.local/bin is in your PATH"


# ==============================================================================
# Documentation & Setup
# ==============================================================================

doc:
	@echo "Generating documentation..."
	cargo doc --no-deps --open

setup:
	@echo "Installing development tools..."
	rustup component add rustfmt clippy
	cargo install cargo-watch
	@echo "Development tools installed successfully!"
