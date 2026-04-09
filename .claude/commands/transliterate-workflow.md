Run the full transliteration pipeline: convert Cyrillic text to Latin in string tables and UI translation, then build the mod.

This is the primary workflow for PS5 users who cannot use Cyrillic font injection.

## Arguments

`$ARGUMENTS` — optional `--credit "Author Name"` flag for third-party translation attribution.

Examples:
- `/project:transliterate-workflow` — run without credits
- `/project:transliterate-workflow --credit "ZoG"` — attribute translation author

## Steps

All paths are relative to the repo root. Cargo commands use `cd tools/ba2-packer &&` prefix.

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
cp src/strings/*.STRINGS src/strings/*.DLSTRINGS src/strings/*.ILSTRINGS build/translit-input/ 2>/dev/null
cp src/interface/translate_en.txt build/translit-input/ 2>/dev/null
```

If step 3 ran, copy from `build/renamed/` instead of `src/strings/`:

```bash
cp build/renamed/*.STRINGS build/renamed/*.DLSTRINGS build/renamed/*.ILSTRINGS build/translit-input/ 2>/dev/null
```

### 5. Transliterate

```bash
cd tools/ba2-packer && cargo run --release -- transliterate \
  --input-dir ../../build/translit-input \
  --output-dir ../../build/transliterated \
  $ARGUMENTS
```

### 6. Stage transliterated files

```bash
cp build/transliterated/*.STRINGS src/strings/ 2>/dev/null
cp build/transliterated/*.DLSTRINGS src/strings/ 2>/dev/null
cp build/transliterated/*.ILSTRINGS src/strings/ 2>/dev/null
cp build/transliterated/translate_en.txt src/interface/ 2>/dev/null
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

Use the `standard-font-translit` profile if the build does not include custom Cyrillic fonts (`fontconfig_en.txt` / `fonts_en.swf`):

```bash
cd tools/ba2-packer && cargo run --release -- validate ../../dist \
  --source-strings ../../src/strings \
  --source-interface ../../src/interface \
  --profile standard-font-translit
```

Report all results. If any validation check fails, list it and suggest a fix.

## Notes

- Do NOT assume `ba2-packer` is in PATH — always use `cargo run --release --` from `tools/ba2-packer/`
- Output directory is `dist/` (not `output/`)
- The `transliterate` command works on **binary** string tables, not JSONL
- If `fonts_en.swf` with Cyrillic glyphs IS available, suggest the `build-mod` agent instead (transliteration is only needed when Cyrillic fonts are unavailable)
- If the user provides `--credit`, a `CREDITS.txt` file will be created in the output
