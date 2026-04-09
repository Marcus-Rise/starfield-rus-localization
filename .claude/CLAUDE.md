# Starfield Russian Translation Mod — Claude Instructions

## Project Overview

Tools and infrastructure for building a Russian localization mod for Starfield on PS5.
Primary platform: **PS5** (via Bethesda Creations).

Detailed technical architecture: `docs/ARCHITECTURE.md`

## Project Decisions

### Tooling
- **Rust only** — no Python/Node.js. Single CLI `tools/ba2-packer` with subcommands: `pack`, `validate`, `rename`
- **Rust dependencies**: `ba2` (bsa-rs), `clap` (derive), `anyhow`, `encoding_rs`

### Copyright
- **Fonts**: Repository MUST NOT contain proprietary fonts (NB Architekt, NB Grotesk, Handwritten_Institute). Only tools and instructions for creating SWF with free fonts (SIL OFL: PT Sans, Noto Sans, etc.) via JPEXS FFDec
- **Translations**: Repository MUST NOT contain text from third-party translations (ZoG/Segnet, etc.). Only tools, format templates, and placeholders. User adds their own translation
- **Game data**: .STRINGS/.DLSTRINGS/.ILSTRINGS and ESM plugins are created from user's own data. Repository contains only `.gitkeep` placeholders
- **`rename` subcommand**: Converts only the user's own copy of a reference mod (`_ru` → `_en`)

### Development Process
- **TDD**: Tests are written before implementation. `cargo test` must pass at every stage
- **Linting**: `cargo fmt` + `cargo clippy -D warnings` — mandatory in CI and before commits
- **Communication**: Code and variable names in English; README and user docs in Russian
- **Branching**: Never commit to main directly, always use PRs

## Rules for Agents

1. **DO NOT** add real translation text from third-party sources
2. **DO NOT** include proprietary content (fonts, game assets)
3. **DO NOT** commit to main directly
4. **ALWAYS** run `cargo test` before committing
5. **ALWAYS** run `cargo fmt --check` before committing
6. **ALWAYS** follow TDD — write tests first, then implementation

## Project Structure

```
.claude/              # Claude Code configuration
  settings.json       # Permissions, hooks
  CLAUDE.md           # These instructions
  agents/             # Custom agents
src/
  strings/            # Placeholders for string tables (binary, Git LFS)
  interface/          # Templates: fontconfig, translate; placeholder for fonts_en.swf
  plugin/             # Placeholder for Spriggit ESM
tools/ba2-packer/     # Rust CLI: pack, validate, rename
docs/                 # Project documentation
.github/workflows/    # CI/CD
```
