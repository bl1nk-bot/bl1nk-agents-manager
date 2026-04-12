# Contributing

## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.
## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check existing issues to see if the problem has already been reported.

When creating a bug report, please include as much detail as possible:

```markdown
**Describe the bug**
A clear description of the bug

**To Reproduce**
Steps to reproduce:
1. Go to '...'
2. Click on '....'
3. See error

**Expected behavior**
What you expected to happen

**Screenshots**
If applicable

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Gemini CLI Version: [e.g., 1.2.3]
- Bl1nk Version: [e.g., 0.2.0]
- Rust Version: [e.g., 1.70.0]
- Python Version: [e.g., 3.10.0]
```

### Suggesting Features

We welcome feature requests! When suggesting a feature, please explain the problem it solves and provide use cases:

```markdown
**Is your feature request related to a problem?**
A clear description of the problem

**Describe the solution you'd like**
What you want to happen

**Describe alternatives you've considered**
Other solutions you've thought about

**Additional context**
Any other relevant information
```

### Pull Requests

We love pull requests! Here's how to contribute code:

1. Fork the repository
2. Create a new branch for your feature or bug fix (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Update documentation as needed
6. Commit your changes using conventional commit format (see below)
7. Push to your fork (`git push origin feature/amazing-feature`)
8. Open a pull request

## Development Setup

### Prerequisites

- Rust 1.70+ (for building the core extension)
- Python 3.8+ (for management scripts)
- Gemini CLI installed and configured
- Git
- Cargo (comes with Rust)

### Setup Instructions

```bash
# Clone the repository
git clone https://github.com/billlzzz18/bl1nk-agents-manager.git
cd bl1nk-agents-manager

# Install Rust components
rustup update

# Setup development tools
just setup

# Build the project
just build

# Run tests
just test
```

### Project Structure

```
bl1nk-agents-manager/
├── crates/
│   ├── core/              # Core library (16 agent modules, 35+ hooks)
│   │   └── src/
│   │       ├── agents/    # Agent system
│   │       ├── hooks/     # Hook system (35+ hooks)
│   │       ├── mcp/      # MCP protocol
│   │       ├── session/   # Session management
│   │       ├── filesystem/ # File operations
│   │       ├── search/    # Conversation search
│   │       ├── projects/  # Project management
│   │       ├── adapters/  # Protocol adapters
│   │       ├── config/   # Configuration
│   │       ├── rpc/      # RPC handling
│   │       └── events/   # Event system
│   └── server/            # HTTP/Rocket server
├── agents/                # Agent definitions (48+ agents)
├── commands/             # CLI command definitions
├── skills/               # AI skill definitions
├── scripts/              # Python management scripts
└── docs/                 # Documentation
```

## Development Workflow

### Branch Naming Convention

- `feature/description` for new features
- `fix/description` for bug fixes
- `docs/description` for documentation updates
- `refactor/description` for refactoring
- `perf/description` for performance improvements
- `test/description` for test additions

### Commit Messages

We follow the conventional commits specification:

```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`

Examples:

```
feat(agent): add new architect agent with XML prompting
fix(cli): resolve issue with agent switching command
docs(readme): update installation instructions
refactor(core): improve agent loading performance
test(core): add unit tests for agent routing
```

## Coding Standards

### Rust Guidelines

- Follow Rust idioms and best practices
- Use `cargo fmt` to format code
- Use `cargo clippy` for linting
- Write comprehensive tests
- Document public APIs with `///` comments
- Use descriptive variable and function names
- Keep functions focused and small
- Follow the principle of least surprise

### Python Guidelines

- Follow PEP 8 style guide
- Use type hints where appropriate
- Write docstrings for functions and classes
- Keep functions focused and small
- Use descriptive variable names

### Best Practices

- Write self-documenting code
- Add comments for complex logic
- Keep functions small and focused
- Follow SOLID principles
- Write tests for new functionality
- Update documentation when making changes

## Testing

### Writing Tests

- Unit tests required for new features
- Integration tests for API changes
- Test edge cases and error conditions

### Running Tests

```bash
# Run all Rust tests
just test

# Run specific test suite
cargo test --package bl1nk-core

# Run Python validation tests
python3 scripts/validate_agents.py

# Run tests with coverage (if available)
cargo test --all-features -- --include-ignored
```

### Test Coverage

- Aim for high test coverage on new functionality
- Test both happy path and error conditions
- Ensure existing tests continue to pass

## Documentation

### Code Documentation

- Document public APIs with comprehensive comments
- Include examples where helpful
- Explain complex algorithms
- Keep documentation up-to-date with code changes

### README Updates

- Update documentation for new features
- Add examples for new functionality
- Update configuration options and usage instructions

## Pull Request Process

### Before Submitting

- Run tests locally (`just test`)
- Run linter (`just clippy`)
- Update documentation as needed
- Self-review your code changes
- Ensure your changes work as expected

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Refactoring
- [ ] Performance improvement

## Related Issues
Fixes #(issue number)

## Testing
- [ ] Tests pass locally
- [ ] Added new tests
- [ ] Updated existing tests
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-reviewed code
- [ ] Commented complex code
- [ ] Updated documentation
- [ ] No new warnings or errors
- [ ] Commits follow conventional format
```

### Review Process

- Maintainers will be assigned to review your PR
- Address feedback promptly and thoroughly
- Maintain discussion until PR is approved
- Squash commits if requested by maintainers

### Merging

- Requires approval from at least one maintainer
- All CI checks must pass
- Any conflicts with the base branch must be resolved
- PR will be merged by a maintainer after approval

## Adding New Agents

If you're adding a new agent to the library:

1. Create a new markdown file in `agents/` directory
2. Include proper YAML frontmatter with required metadata:

```yaml
---
id: my-new-agent
name: My New Agent
description: A brief description of what this agent does
category: utility
cost: FREE
triggers: ["specific", "keywords", "that", "trigger", "this", "agent"]
---
```

1. Add the system prompt content after the frontmatter
2. Add agent to `agents/agents.json` registry
3. Run `just validate-agents` to ensure your agent is properly formatted
4. Test the agent by running `/system-agent` and verifying it appears in the list

## Adding New Hooks

If you're adding a new hook to the system:

1. Create a new module in `crates/core/src/hooks/`
2. Implement the hook interface:

```rust
pub struct MyHook {
    // Hook state
}

impl MyHook {
    pub fn new() -> Self {
        Self { /* ... */ }
    }
    
    pub async fn process(&self, context: &Context) -> Result<()> {
        // Hook logic
        Ok(())
    }
}
```

1. Export in `crates/core/src/hooks/mod.rs`:

```rust
pub mod my_hook;
pub use my_hook::{create_my_hook, MyHook};
```

1. Add tests for the hook

## Community

### Getting Help

- **Issues**: For bug reports and feature requests, use the GitHub Issues tab
- **Discussions**: For questions and community discussions
- **Documentation**: Check the README and docs/ directory

### Recognition

We appreciate all contributions, big and small. Contributors will be acknowledged in the project's README and release notes.

## Release Process (For Maintainers)

1. Update version in `Cargo.toml` files
2. Update `agents/agents.json` version
3. Update changelog with notable changes
4. Create a new git tag with the version number
5. Create a GitHub release with the tag

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project (MIT License).

---

Thank you for contributing to Bl1nk Agents Manager! Your efforts help make the AI agent ecosystem better for everyone.
