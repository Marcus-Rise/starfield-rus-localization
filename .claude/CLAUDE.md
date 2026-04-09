# Starfield Russian Translation Mod — Claude Instructions

## Project Overview

Tools and infrastructure for building a Russian localization mod for Starfield on PS5.
Primary platform: **PS5** (via Bethesda Creations).

Detailed technical architecture: `docs/ARCHITECTURE.md`

## Project Decisions

### Tooling
- **Rust only** — no Python/Node.js. Single CLI `tools/ba2-packer` with subcommands: `pack`, `validate`, `rename`, `extract`, `repack`, `create-esm`, `transliterate`
- **Rust dependencies**: `ba2` (bsa-rs), `clap` (derive), `anyhow`, `encoding_rs`, `serde`/`serde_json`

### Copyright
- **Fonts**: Repository MUST NOT contain proprietary fonts (NB Architekt, NB Grotesk, Handwritten_Institute). Only tools and instructions for creating SWF with free fonts (SIL OFL: PT Sans, Noto Sans, etc.) via JPEXS FFDec
- **Translations**: Repository MUST NOT contain text from third-party translations (ZoG/Segnet, etc.). Only tools, format templates, and placeholders. User adds their own translation
- **Game data**: .STRINGS/.DLSTRINGS/.ILSTRINGS and ESM plugins are created from user's own data. Repository contains only `.gitkeep` placeholders
- **`rename` subcommand**: Converts only the user's own copy of a reference mod (`_ru` → `_en`)

### PS5 Platform Constraints

> Full technical details: `docs/ARCHITECTURE.md`

1. **BA2 archives only** — PS5 cannot load loose files; all content must be packed into BA2 and delivered via Creations
2. **No INI modifications** — any INI changes on PS5 cause console hangs; crash logs are sent to Sony and risk hardware ban
3. **No native Cyrillic** — engine fonts lack Cyrillic glyphs; must inject via `fonts_en.swf` using JPEXS FFDec
4. **BA2 revert on updates** — game updates reset custom BA2 files; mod must be re-applied after each patch
5. **Never set PS5 language to Russian** — causes console hang
6. **Creations is the only distribution path** — no sideloading on PS5
7. **Transliteration as fallback** — PS5 community accepts Cyrillic→Latin transliterated text when font injection is not viable

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
7. **ALWAYS** update documentation when adding/modifying CLI subcommands:
   - `README.md` (command table, examples)
   - `CLAUDE.md` (root — subcommand list)
   - `.claude/CLAUDE.md` (tooling section, project structure)
   - `docs/WORKFLOW.md` (if workflow changes)
   - `docs/DOR_DOD.md` (if DoD criteria change)
   - `CONTRIBUTING.md` (if user-facing workflow changes)
   - `.claude/agents/build-mod.md` (if build pipeline changes)
8. **ALWAYS** run documentation self-review before completing a PR:
   - Verify all docs reference the current set of subcommands
   - Verify examples match current CLI interface (`ba2-packer --help`)
   - Verify agents (`.claude/agents/`) reflect the actual build pipeline

## Anti-patterns (Do NOT)

> Copyright, TDD, and branching anti-patterns are already covered in Rules for Agents above.

**Platform:**
- No `sResourceStartUpArchiveList` modifications for PS5 — causes hangs
- No loose file workflows — always pack into BA2
- No PS5 system language changes in testing guides or documentation

**Code:**
- No `unwrap()` without a comment explaining why the value is guaranteed — use `anyhow::Context`
- No new Cargo dependencies without first checking if existing deps cover the need
- No changes to binary STRINGS/DLSTRINGS/ILSTRINGS serialization without round-trip tests
- No modifications to `string_table.rs` serialization without format-level tests against fixture files

**Process:**
- No refactoring unrelated code in feature PRs — keep PRs focused on a single concern

## Project Structure

```
.claude/              # Claude Code configuration
  settings.json       # Permissions, hooks
  CLAUDE.md           # These instructions
  agents/             # Custom agents
    add-translation.md
    build-mod.md
    lint-and-test.md
    onboarding.md
    publish-to-creations.md
    transliterate-mod.md
  commands/            # Slash commands: full-validate, transliterate-workflow, publish-check
src/
  strings/            # Placeholders for string tables (binary, Git LFS)
  interface/          # Templates: fontconfig, translate; placeholder for fonts_en.swf
  plugin/             # Placeholder for Spriggit ESM
tools/ba2-packer/     # Rust CLI: pack, validate, rename, extract, repack, create-esm, transliterate
docs/                 # Project documentation
.github/workflows/    # CI/CD
```
