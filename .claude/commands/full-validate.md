Run full validation of the built mod artifacts in `dist/`.

## Arguments

`$ARGUMENTS` — optional extra flags passed directly to `ba2-packer validate`.
By default validates `../../dist` with `--source-strings ../../src/strings --source-interface ../../src/interface`.

Examples:
- `/project:full-validate` — validate `dist/` with source dirs
- `/project:full-validate --require-credits` — also enforce CREDITS.txt

## Steps

1. **Build the CLI tool** (skip if already built):
   ```bash
   cd tools/ba2-packer && cargo build --release
   ```

2. **Run validation**:
   ```bash
   cd tools/ba2-packer && cargo run --release -- validate ../../dist --source-strings ../../src/strings --source-interface ../../src/interface $ARGUMENTS
   ```

3. **Report results** — summarize which checks passed, failed, or produced warnings. If any checks failed, list them clearly and suggest fixes.

## Notes

- Do NOT assume `ba2-packer` is in PATH — always use `cargo run --release --` from `tools/ba2-packer/`
- Output directory is `dist/` (not `output/`)
- The validate subcommand runs 13 checks + 1 warning (ESM flags, string files, UI translation encoding, font config, SWF magic, BA2 headers, size limits, credits)
- If `dist/` does not exist or is empty, tell the user to run the build first (use the `build-mod` or `transliterate-mod` agent)
