Run the full transliteration pipeline: convert Cyrillic text to Latin in string tables and UI translation, then build the mod.

This is the primary workflow for PS5 users who cannot use Cyrillic font injection.

## Arguments

`$ARGUMENTS` — optional `--credit "Author Name"` flag for third-party translation attribution.

Examples:
- `/project:transliterate-workflow` — run without credits
- `/project:transliterate-workflow --credit "ZoG"` — attribute translation author

## Steps

Execute all commands from `tools/ba2-packer/`.

### 1. Build the CLI tool

```bash
cd tools/ba2-packer && cargo build --release
```

### 2. Check prerequisites

Verify that `src/strings/` contains real string table files (`.STRINGS`, `.DLSTRINGS`, `.ILSTRINGS`), not just `.gitkeep`. If only `.gitkeep` exists, stop and tell the user to add translation files first (see `add-translation` agent).

Also verify that `src/interface/translate_en.txt` exists.

### 3. Rename `_ru` to `_en` (conditional)

Check if any files in `src/strings/` have `_ru` suffix. If so, rename them:

```bash
cd tools/ba2-packer && cargo run --release -- rename \
  --input-dir ../../src/strings \
  --output-dir ../../build/renamed
```

If files already have `_en` suffix, skip this step.

### 4. Prepare transliteration input

```bash
mkdir -p build/translit-input
```

Copy string tables (from `build/renamed/` if step 3 ran, otherwise from `src/strings/`) and `src/interface/translate_en.txt` into `build/translit-input/`.

### 5. Transliterate

```bash
cd tools/ba2-packer && cargo run --release -- transliterate \
  --input-dir ../../build/translit-input \
  --output-dir ../../build/transliterated \
  $ARGUMENTS
```

### 6. Stage transliterated files

Copy output back into source directories:

```bash
cp build/transliterated/*.STRINGS build/transliterated/*.DLSTRINGS build/transliterated/*.ILSTRINGS src/strings/
cp build/transliterated/translate_en.txt src/interface/
```

### 7. Create ESM plugin

```bash
cd tools/ba2-packer && cargo run --release -- create-esm \
  --output ../../dist/StarfieldRussian.esm
```

### 8. Pack BA2 archives

```bash
cd tools/ba2-packer && cargo run --release -- pack \
  --input-strings ../../src/strings \
  --input-interface ../../src/interface \
  --output-dir ../../dist
```

### 9. Validate

```bash
cd tools/ba2-packer && cargo run --release -- validate ../../dist
```

Report all results. If any validation check fails, list it and suggest a fix.

## Notes

- Do NOT assume `ba2-packer` is in PATH — always use `cargo run --release --` from `tools/ba2-packer/`
- Output directory is `dist/` (not `output/`)
- The `transliterate` command works on **binary** string tables, not JSONL
- If `fonts_en.swf` with Cyrillic glyphs IS available, suggest the `build-mod` agent instead (transliteration is only needed when Cyrillic fonts are unavailable)
- If the user provides `--credit`, a `CREDITS.txt` file will be created in the output
