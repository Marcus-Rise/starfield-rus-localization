# Starfield PS5 Russian Translation Mod

Free Creations mod that provides Russian localization tools for Starfield on PlayStation 5.

## Quick Reference

- **Claude instructions & project decisions**: `.claude/CLAUDE.md`
- **Technical architecture**: `docs/ARCHITECTURE.md`
- **DoR/DoD**: `docs/DOR_DOD.md`
- **Publishing guides**: `docs/PUBLISH_CREATIONS.md`, `docs/PUBLISH_NEXUS.md`

## Key Rules

- **Rust only** — single CLI `tools/ba2-packer` with subcommands: `pack`, `validate`, `rename`, `extract`, `repack`, `create-esm`, `transliterate`
- **No proprietary content** — no copyrighted fonts, no third-party translations
- **TDD** — tests first, `cargo test` must pass at every stage
- **Linting** — `cargo fmt` + `cargo clippy -D warnings` mandatory

See `.claude/CLAUDE.md` for full details.
