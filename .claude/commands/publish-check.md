Pre-publish readiness check before uploading the mod to Bethesda Creations.

This is a read-only diagnostic — it does not modify any files.

## Steps

### 1. Check required artifacts

Verify that `dist/` contains all three required files:
- `StarfieldRussian.esm`
- `StarfieldRussian - Main.ba2`
- `StarfieldRussian - Interface.ba2`

If any are missing, report which ones and tell the user to run the `build-mod` or `transliterate-mod` agent first.

### 2. Check artifact sizes

```bash
ls -lh dist/StarfieldRussian.esm "dist/StarfieldRussian - Main.ba2" "dist/StarfieldRussian - Interface.ba2"
```

- **Hard limit**: total MUST be < 2 GB. If exceeded — stop, Creations will reject the upload.
- **Warning**: if total > 100 MB, warn about download size impact for end users.

### 3. Run validation

```bash
cd tools/ba2-packer && cargo build --release && cargo run --release -- validate ../../dist --source-strings ../../src/strings --source-interface ../../src/interface
```

All checks must pass. Report any failures.

### 4. Check credits (if third-party translation)

Check if `dist/CREDITS.txt` exists. If it does, display its contents. If it does not, remind the user:
- If using a third-party translation, the build must use `--credit "Author Name"`
- The mod description on Creations must credit the translation author

### 5. PS5 font caveat

Print this warning:

> **PS5**: Cyrillic font loading via Creations is unverified. If Creations does not register `Interface.ba2` in `sResourceStartUpArchiveList`, Cyrillic text will not render. Manual INI edits on PS5 cause system hangs (risk of hardware ban). Consider the transliterated variant as a fallback.

### 6. Summary

Print a go/no-go summary:

- All 3 artifacts present?
- Total size < 2 GB?
- Validation passed?
- Credits present (if applicable)?

If all checks pass: **Ready to publish.** Direct the user to the `publish-to-creations` agent or `docs/PUBLISH_CREATIONS.md` for the CK2 upload workflow.

If any check fails: **Not ready.** List what needs to be fixed.

## Notes

- Do NOT assume `ba2-packer` is in PATH — always use `cargo run --release --` from `tools/ba2-packer/`
- Output directory is `dist/` (not `output/`)
- This command does NOT upload anything — it only checks readiness
- The actual CK2 upload is a manual Windows GUI process (see `publish-to-creations` agent)
