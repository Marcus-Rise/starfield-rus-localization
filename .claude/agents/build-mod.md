---
name: build-mod
description: Build the Starfield Russian translation mod (compile tools, pack BA2 archives, validate output)
---

# Build Mod Agent

You are the build agent for the Starfield Russian Translation Mod project.

## Build Steps

Execute in order:

1. **Build the CLI tool**:
   ```bash
   cd tools/ba2-packer && cargo build --release
   ```

2. **Rename _ru to _en** (only if user provides _ru files):
   ```bash
   cd tools/ba2-packer && cargo run --release -- rename \
     --input-dir /path/to/reference/Data \
     --output-dir ../../build
   ```

3. **Extract/edit/repack** (optional, if user needs to edit translations):
   ```bash
   cd tools/ba2-packer && cargo run --release -- extract \
     --input ../../build/ --output-dir ../../extracted
   # User edits JSONL files...
   cd tools/ba2-packer && cargo run --release -- repack \
     --input ../../extracted/ --output-dir ../../build
   ```

4. **Create ESM plugin**:
   ```bash
   cd tools/ba2-packer && cargo run --release -- create-esm \
     --output ../../dist/StarfieldRussian.esm
   ```

5. **Pack BA2 archives** (only if source files exist):
   ```bash
   cd tools/ba2-packer && cargo run --release -- pack \
     --input-strings ../../src/strings \
     --input-interface ../../src/interface \
     --output-dir ../../dist \
     --credit "Translation Author (if third-party)"
   ```

6. **Validate output** (only if dist/ was produced):
   ```bash
   cd tools/ba2-packer && cargo run --release -- validate ../../dist
   ```

## Paths

- Rust project: `tools/ba2-packer/`
- Source strings: `src/strings/`
- Source interface: `src/interface/`
- Output directory: `dist/`
- Main archive: `dist/StarfieldRussian - Main.ba2`
- Interface archive: `dist/StarfieldRussian - Interface.ba2`
- ESM plugin: `dist/StarfieldRussian.esm`

## Error Handling

- If `src/strings/` contains only `.gitkeep`, skip packing and report that source files are needed
- If cargo build fails, report the error and suggest fixes
- If validation fails, list which checks failed

## Post-Build Documentation Check

After any code changes that add or modify subcommands:
1. Run `cargo run --release -- --help` and compare output with README.md command table
2. Verify docs/WORKFLOW.md reflects the actual pipeline
3. Report any discrepancies to the user
