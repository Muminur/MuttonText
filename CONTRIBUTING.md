# Contributing to MuttonText

Thank you for your interest in contributing to MuttonText! This document provides guidelines for contributing to the project.

## How to Report Bugs

Before submitting a bug report:
1. Check the [existing issues](https://github.com/Muminur/MuttonText/issues) to avoid duplicates
2. Verify the bug exists in the latest version
3. Test with all combos and groups disabled to isolate the issue

When submitting a bug report, include:
- Operating system and version (Linux distro, macOS version)
- Desktop environment (for Linux: X11/Wayland, GNOME/KDE/etc.)
- MuttonText version
- Steps to reproduce the issue
- Expected vs. actual behavior
- Relevant logs from `~/.config/muttontext/logs/` (Linux) or `~/Library/Application Support/muttontext/logs/` (macOS)

## How to Suggest Features

Feature requests are welcome! Please:
1. Check [existing feature requests](https://github.com/Muminur/MuttonText/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)
2. Describe the problem you're trying to solve
3. Explain your proposed solution
4. Consider how it fits with the project's privacy-first, local-only philosophy
5. Note if the feature exists in Beeftext (we aim for feature parity)

## Development Setup

See [README.md](README.md#build-from-source) for detailed platform-specific setup instructions.

Quick setup:
```bash
git clone https://github.com/Muminur/MuttonText.git
cd MuttonText
npm install
cargo check
npm run tauri dev
```

## Branch Naming Convention

Use descriptive branch names with task references:
```
feature/MT-XXX-description
bugfix/MT-XXX-description
hotfix/MT-XXX-description
```

Examples:
- `feature/MT-101-combo-crud-operations`
- `bugfix/MT-245-clipboard-race-condition`

## Commit Message Format

Follow the conventional commits format:
```
<type>(<scope>): <description>

[optional body]
```

Types:
- `feat` - New feature
- `fix` - Bug fix
- `refactor` - Code refactoring
- `test` - Test additions or changes
- `chore` - Maintenance tasks
- `docs` - Documentation updates

Examples:
- `feat(combo-manager): implement CRUD operations for combos`
- `fix(input-manager): resolve keyboard hook memory leak on Linux`
- `refactor(variable-eval): simplify date formatting logic`

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes following the code style guidelines
3. Write or update tests to cover your changes
4. Run the pre-push validation script: `./scripts/pre-push.sh`
5. Commit with descriptive messages
6. Push your branch and create a pull request
7. Ensure all CI checks pass
8. Wait for code review and address feedback

Pull requests must:
- Reference the related issue or task number
- Include tests for new functionality
- Pass all CI checks
- Follow the project's code style
- Not introduce new warnings or errors

## Code Style

### Rust
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

Follow Rust conventions:
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and structs
- Document public APIs with doc comments
- Handle errors explicitly (no `unwrap` in production code)

### TypeScript/React
```bash
# Run linter
npm run lint

# Fix auto-fixable issues
npm run lint:fix

# Type check
npm run typecheck
```

Follow TypeScript conventions:
- Use `camelCase` for functions and variables
- Use `PascalCase` for components and types
- Use functional components with TypeScript interfaces for props
- Prefer hooks over class components

Code is formatted automatically with Prettier.

## Testing Requirements

We follow Test-Driven Development (TDD):
1. Write failing test
2. Implement minimum code to pass
3. Refactor if needed

### Running Tests

```bash
# All tests
npm run test:all

# Rust tests
cargo test --workspace

# Frontend tests
npm run test

# E2E tests
npm run test:e2e
```

### Coverage Requirements
- Rust backend: 80% minimum
- React frontend: 75% minimum
- Critical user flows: 100%

All pull requests must include tests. Changes without tests will not be merged.

## Questions?

If you have questions not covered here:

- Open a [GitHub Discussion](https://github.com/Muminur/muttontext/discussions)
- Ask in pull request comments

Thank you for contributing to MuttonText!
