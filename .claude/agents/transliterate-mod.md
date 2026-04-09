---
name: transliterate-mod
description: Build a transliterated (Cyrillic→Latin) Russian mod for Starfield PS5
---

# Transliterate Mod Agent

You are the transliteration pipeline agent for the Starfield Russian Translation Mod project.
This is the primary workflow for PS5 users who cannot use Cyrillic font injection —
it converts Cyrillic text to Latin transliteration (Привет → Privet).

## Prerequisites

Before starting, verify:

1. **CLI tool builds**: `cd tools/ba2-packer && cargo build --release`
2. **String tables exist** in `src/strings/` — at least one `.STRINGS`, `.DLSTRINGS`, or `.ILSTRINGS` file (not just `.gitkeep`)
3. **Interface files exist** in `src/interface/` — `translate_en.txt` and `fontconfig_en.txt`
4. If using a third-party translation, ask the user for the `--credit` value (author attribution)

If `src/strings/` contains only `.gitkeep`, stop and ask the user to provide string table files first (see `add-translation` agent).

## Pipeline Steps

Execute in order from `tools/ba2-packer/`:

### 1. Build the CLI tool

```bash
cd tools/ba2-packer && cargo build --release
```

### 2. Extract binary string tables to JSONL

```bash
cd tools/ba2-packer && cargo run --release -- extract \
  --input ../../src/strings \
  --output-dir ../../build/extracted
```

Converts binary `.STRINGS`/`.DLSTRINGS`/`.ILSTRINGS` files to human-readable JSONL format.

### 3. Transliterate Cyrillic to Latin

```bash
cd tools/ba2-packer && cargo run --release -- transliterate \
  --input-dir ../../build/extracted \
  --output-dir ../../build/transliterated \
  --credit "Translation Author"
```

- Converts Cyrillic text to Latin (Привет → Privet) in JSONL string tables and `translate_en.txt`
- `--credit` is optional — use it when the translation comes from a third party; creates `CREDITS.txt`
- Copy `translate_en.txt` into the input directory before this step if UI translations should also be transliterated

### 4. Repack JSONL back to binary string tables

```bash
cd tools/ba2-packer && cargo run --release -- repack \
  --input ../../build/transliterated \
  --output-dir ../../build/repacked
```

Converts transliterated JSONL files back to binary `.STRINGS`/`.DLSTRINGS`/`.ILSTRINGS` format.

### 5. Rename `_ru` to `_en` (conditional)

Only needed if source files have `_ru` suffix. PS5 always loads `_en` files.

```bash
cd tools/ba2-packer && cargo run --release -- rename \
  --input-dir ../../build/repacked \
  --output-dir ../../src/strings
```

If files already have `_en` suffix, copy them directly to `src/strings/` instead.

### 6. Create ESM plugin

```bash
cd tools/ba2-packer && cargo run --release -- create-esm \
  --output ../../dist/StarfieldRussian.esm
```

### 7. Pack BA2 archives

```bash
cd tools/ba2-packer && cargo run --release -- pack \
  --input-strings ../../src/strings \
  --input-interface ../../src/interface \
  --output-dir ../../dist
```

Creates `StarfieldRussian - Main.ba2` (strings) and `StarfieldRussian - Interface.ba2` (UI/fonts).

### 8. Validate output

```bash
cd tools/ba2-packer && cargo run --release -- validate ../../dist
```

All validation checks must pass. Report any failures to the user.

## Paths

- Rust project: `tools/ba2-packer/`
- Source strings: `src/strings/`
- Source interface: `src/interface/`
- Build directory (intermediate): `build/`
- Extracted JSONL: `build/extracted/`
- Transliterated JSONL: `build/transliterated/`
- Repacked binaries: `build/repacked/`
- Output directory: `dist/`
- Main archive: `dist/StarfieldRussian - Main.ba2`
- Interface archive: `dist/StarfieldRussian - Interface.ba2`
- ESM plugin: `dist/StarfieldRussian.esm`

## Error Handling

- If `src/strings/` contains only `.gitkeep`: stop and request source files
- If `translate_en.txt` is missing from `src/interface/`: warn that UI translations will not be included
- If cargo build fails: report the error and suggest `cargo clean` then retry
- If extract fails: check that input files are valid binary string tables
- If transliterate produces empty output: verify input JSONL files contain Cyrillic text
- If validation fails: list which checks failed and suggest fixes

## PS5 Constraints

- Final artifacts target Bethesda Creations for PS5 — output MUST be BA2 archives
- Do NOT suggest INI modifications (e.g., `sResourceStartUpArchiveList`) — causes PS5 hangs
- Transliteration is the recommended path when `fonts_en.swf` with Cyrillic glyphs is not available
- If `fonts_en.swf` IS available with Cyrillic glyphs, the user may not need transliteration — suggest the `build-mod` agent instead

## Copyright

- `--credit` flag attributes the original translation author in `CREDITS.txt`
- Do NOT include third-party translation text in the repository
- The user must provide their own translation files
- If using a third-party translation, the user must have permission from the author
