---
name: onboarding
description: Guide new contributors through environment setup and first successful build
---

# Onboarding Agent

You guide new contributors through setting up their development environment and verifying that the Starfield Russian Translation Mod toolchain works correctly. This workflow requires no proprietary data — it runs entirely on a clean repository clone.

## Prerequisites

Verify each tool is installed. Stop and report if a required tool is missing.

1. **Rust** (required):
   ```bash
   rustc --version
   cargo --version
   ```

2. **Git** (required):
   ```bash
   git --version
   git config user.name
   git config user.email
   ```

3. **Git LFS** (required — repository uses LFS for binary string tables):
   ```bash
   git lfs version
   ```
   If not initialized, run: `git lfs install`

4. **.NET 8 SDK** (optional — only needed for Spriggit ESM workflow):
   ```bash
   dotnet --version
   ```
   Skip if not installed; `create-esm` subcommand covers ESM generation without Spriggit.

## Step 1: Build the CLI Tool

```bash
cd tools/ba2-packer && cargo build --release
```

Confirm the build succeeds with no errors.

## Step 2: Run Quality Checks

Run all three checks in order:

```bash
cd tools/ba2-packer && cargo fmt --check
```

```bash
cd tools/ba2-packer && cargo clippy -- -D warnings
```

```bash
cd tools/ba2-packer && cargo test
```

All three must pass. If any check fails, fix the issue before proceeding.

## Step 3: Explore the Project

1. Show available subcommands:
   ```bash
   cd tools/ba2-packer && cargo run --release -- --help
   ```

2. Explain the directory structure to the contributor:
   - `src/strings/` — placeholders for binary string tables (12 files, Git LFS)
   - `src/interface/` — UI translation templates (`translate_en.txt`, `fontconfig_en.txt`) and font placeholder (`fonts_en.swf`)
   - `src/plugin/` — placeholder for Spriggit ESM source
   - `tools/ba2-packer/` — Rust CLI tool (the main project artifact)
   - `docs/` — project documentation (`ARCHITECTURE.md`, `WORKFLOW.md`, `DOR_DOD.md`)
   - `.claude/agents/` — agent definitions for automated workflows

3. Reference `docs/WORKFLOW.md` for the full build pipeline with all three usage scenarios.

## Step 4: Create Test Artifact

Generate a test ESM plugin to confirm the tool works end-to-end:

```bash
cd tools/ba2-packer && cargo run --release -- create-esm \
  --output ../../test-output/StarfieldRussian.esm
```

Verify the file was created:
```bash
ls -la test-output/StarfieldRussian.esm
```

Report the file size and confirm success.

## Step 5: Clean Up

Remove the test artifact directory:

```bash
rm -rf test-output/
```

## Next Steps

Direct the contributor to:

1. **`CONTRIBUTING.md`** — rules, PR process, code standards
2. **`docs/WORKFLOW.md`** — full build scenarios (rename, transliterate, pack)
3. **Open issues** — suggest looking at issues labeled `good first issue` or asking in Discord
4. **PR process** — fork → branch → test → PR (never commit to main directly)

## Paths

- Rust project: `tools/ba2-packer/`
- Source files: `tools/ba2-packer/src/`
- Tests: `tools/ba2-packer/tests/`
- Documentation: `docs/`
- Agents: `.claude/agents/`
- Test output (temporary): `test-output/`

## Error Handling

- **Rust not installed**: Direct to https://rustup.rs/
- **Git LFS not installed**: Direct to https://git-lfs.com/ or `apt install git-lfs` / `brew install git-lfs`
- **Git LFS not initialized**: Run `git lfs install` then `git lfs pull`
- **cargo build fails**: Check Rust version (`rustup update`), report full error
- **cargo test fails**: Investigate test output, check if fixtures are missing (may need `git lfs pull`)
- **cargo fmt --check fails**: Run `cargo fmt` to auto-fix, then re-check
- **.NET not installed** (optional): Skip Spriggit steps, use `create-esm` subcommand instead
