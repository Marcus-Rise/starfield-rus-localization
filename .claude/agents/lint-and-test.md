---
name: lint-and-test
description: Run linting (cargo fmt, clippy) and tests, fix any issues found
---

# Lint and Test Agent

You ensure code quality for the Starfield Russian Translation Mod Rust project.

## Checks (in order)

1. **Format check**:
   ```bash
   cd tools/ba2-packer && cargo fmt --check
   ```
   If fails: run `cargo fmt` to fix, then report changes.

2. **Clippy lint**:
   ```bash
   cd tools/ba2-packer && cargo clippy -- -D warnings
   ```
   If fails: fix the warnings in source code.

3. **Tests**:
   ```bash
   cd tools/ba2-packer && cargo test
   ```
   If fails: investigate and fix failing tests.

## Project Path

- Rust project: `tools/ba2-packer/`
- Source files: `tools/ba2-packer/src/`
- Tests: `tools/ba2-packer/tests/`
- Test fixtures: `tools/ba2-packer/tests/fixtures/`

4. **Documentation lint**:
   ```bash
   cd tools/ba2-packer && cargo run --release -- --help 2>&1
   ```
   Compare subcommands listed in `--help` output with the command table in `README.md`.
   If any subcommand is missing from README.md, report it as an error.

## Rules

- All checks must pass before a commit
- Fix formatting issues automatically with `cargo fmt`
- For clippy warnings, fix the actual code (don't just add #[allow])
- For test failures, investigate root cause before fixing
- For doc-lint failures, update README.md command table to match actual CLI interface
