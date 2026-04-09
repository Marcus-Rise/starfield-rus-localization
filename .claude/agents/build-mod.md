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

2. **Pack BA2 archives** (only if source files exist):
   ```bash
   cd tools/ba2-packer && cargo run --release -- pack \
     --input-strings ../../src/strings \
     --input-interface ../../src/interface \
     --output-dir ../../dist
   ```

3. **Validate output** (only if dist/ was produced):
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
