# ==============================================================================
# Makefile for BL1NK Agents Manager (v1.7.0)
# ==============================================================================

.PHONY: help build build-bundled run dev test check fmt clippy clean install doc setup lint lint-md md-fix all-check \
        parallel parallel-verbose agents-check agents-list skills-check skills-list review \
        bump-patch bump-minor bump-major changelog commitlint release

# --- Default Behavior ---
all: build

# --- Help & Documentation ---
help:
	@echo "🤖 BL1NK Agents Manager v1.7.0 - Development Commands"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Build Targets:"
	@echo "  build                 - Build standard release binary"
	@echo "  build-bundled         - Build with bundled PMAT support"
	@echo "  release               - Build Rust binary and generate universal distribution (dist/)"
	@echo ""
	@echo "Agent & Skill Management:"
	@echo "  agents-list           - List all discovered agents and skills"
	@echo "  agents-check          - Validate schemas for all agents and skills"
	@echo "  skills-list           - Alias for agents-list"
	@echo "  skills-check          - Alias for agents-check"
	@echo ""
	@echo "Quality Checks:"
	@echo "  test                  - Run all Rust tests"
	@echo "  fmt                   - Format Rust code"
	@echo "  clippy                - Run clippy linter"
	@echo "  lint-md               - Lint Markdown documentation"
	@echo "  md-fix                - Auto-fix Markdown formatting"
	@echo "  parallel              - Run fmt + clippy + test + agents in PARALLEL (fast)"
	@echo ""
	@echo "Development:"
	@echo "  run                   - Run the orchestrator"
	@echo "  dev                   - Watch mode with auto-reload"
	@echo "  review                - Run code review on changes"
	@echo ""
	@echo "Release:"
	@echo "  bump-patch            - v1.7.0 -> v1.7.1"
	@echo "  bump-minor            - v1.7.0 -> v1.8.0"
	@echo "  bump-major            - v1.7.0 -> v2.0.0"
	@echo "  changelog             - Update CHANGELOG.md"

# ==============================================================================
# Build & Run
# ==============================================================================

build:
	@echo "🚀 Building release binary..."
	cargo build --release

build-bundled:
	@echo "📦 Building with bundled PMAT..."
	cargo build --release --features bundle-pmat

release: build
	@echo "🏗️ Generating universal distribution..."
	@pip install toml pyyaml --quiet
	@python3 scripts/build-release.py
	@echo "🎁 Release distribution ready in dist/"

run:
	@./target/release/bl1nk-agents-manager

dev:
	cargo watch -x 'run --features bundle-pmat'

# ==============================================================================
# Agent & Skill Management
# ==============================================================================

agents-list skills-list:
	@python3 scripts/agent_manager.py list

agents-check skills-check:
	@python3 scripts/validate_agents.py

# ==============================================================================
# Quality & Linting
# ==============================================================================

test:
	cargo test --all-features

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-features -- -D warnings

lint-md:
	@npx -y markdownlint-cli2 "**/*.md"

md-fix:
	@npx -y markdownlint-cli2 "**/*.md" --fix

parallel:
	@bash scripts/parallel-check.sh

parallel-verbose:
	@bash scripts/parallel-check.sh --verbose

all-check: fmt clippy test agents-check lint-md
	@echo "✅ All checks passed!"

# ==============================================================================
# Maintenance
# ==============================================================================

clean:
	cargo clean
	rm -rf target/check-logs
	rm -rf dist/

bump-patch:
	@bash scripts/bumpversion.sh patch

bump-minor:
	@bash scripts/bumpversion.sh minor

bump-major:
	@bash scripts/bumpversion.sh major

changelog:
	@bash scripts/generate-changelog.sh

review:
	@bash scripts/review.sh

setup:
	rustup component add rustfmt clippy
	cargo install cargo-watch
	@echo "✅ Development environment ready!"
