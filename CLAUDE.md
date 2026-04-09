# Starfield PS5 Russian Translation Mod

## Project Goal

Build a free Creations mod that translates Starfield into Russian on PlayStation 5.
Primary platform: **PS5** (via Bethesda Creations).
Secondary: PC (Nexus Mods), Xbox (Creations).

## Problem Statement

Starfield on PS5 has no Russian language support. The game's Scaleform GFx UI engine uses SWF font libraries (`fonts_en.swf`) containing only Latin glyphs. When Cyrillic codepoints are encountered, the engine renders empty rectangles ("squares").

The solution requires:
1. Replacing font SWF files with Cyrillic-capable versions
2. Replacing string table files with Russian translations
3. Replacing UI translation file with Russian text
4. Packaging everything as a Creations-compatible mod (ESM + BA2)

## Repository Structure

```
starfield-russian/
├── CLAUDE.md                         # This file — project context for AI agents
├── README.md                         # Public readme (install instructions, credits)
├── CONTRIBUTING.md                   # How to contribute translations
├── LICENSE                           # MIT or similar
├── .github/
│   └── workflows/
│       ├── build.yml                 # CI: build ESM + BA2 on every push/PR
│       ├── release.yml               # CD: create GitHub Release on tag push
│       └── validate.yml              # PR check: format, structure, completeness
├── src/
│   ├── strings/                      # Translated string tables (binary, git-lfs)
│   │   ├── starfield_en.STRINGS
│   │   ├── starfield_en.DLSTRINGS
│   │   ├── starfield_en.ILSTRINGS
│   │   ├── blueprintships-starfield_en.STRINGS
│   │   ├── blueprintships-starfield_en.DLSTRINGS
│   │   ├── blueprintships-starfield_en.ILSTRINGS
│   │   ├── constellation_en.STRINGS
│   │   ├── constellation_en.DLSTRINGS
│   │   ├── constellation_en.ILSTRINGS
│   │   ├── oldmars_en.STRINGS
│   │   ├── oldmars_en.DLSTRINGS
│   │   └── oldmars_en.ILSTRINGS
│   ├── interface/                    # Font and UI files
│   │   ├── fonts_en.swf              # Cyrillic-patched font library (git-lfs)
│   │   ├── fontconfig_en.txt         # Font mapping config
│   │   └── translate_en.txt          # UI string translations (UTF-16LE BOM)
│   └── plugin/                       # ESM plugin source (Spriggit YAML/JSON)
│       └── StarfieldRussian/
│           └── ...                   # Spriggit-serialized plugin records
├── tools/
│   └── ba2-packer/                   # Rust CLI wrapper around bsa-rs
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── scripts/
│   ├── validate.py                   # Post-build validation
│   └── rename_ru_to_en.py            # One-time: convert reference mod _ru → _en
├── docs/
│   ├── PUBLISH_CREATIONS.md          # Step-by-step guide for co-author (CK2 upload)
│   └── PUBLISH_NEXUS.md              # Self-service Nexus Mods upload guide
├── .gitattributes                    # Git LFS rules for binary files
└── .gitignore                        # dist/, *.ba2 build outputs, etc.
```

### Git LFS
Binary files tracked via Git LFS (configured in `.gitattributes`):
```
*.STRINGS filter=lfs diff=lfs merge=lfs -text
*.DLSTRINGS filter=lfs diff=lfs merge=lfs -text
*.ILSTRINGS filter=lfs diff=lfs merge=lfs -text
*.swf filter=lfs diff=lfs merge=lfs -text
*.ba2 filter=lfs diff=lfs merge=lfs -text
*.esm filter=lfs diff=lfs merge=lfs -text
```

### Build Output (not in repo, produced by CI)
```
dist/
├── StarfieldRussian.esm
├── StarfieldRussian - Main.ba2
└── StarfieldRussian - Interface.ba2
```

## Key Technical Details

### String Tables
- Format: binary, documented at https://en.uesp.net/wiki/Skyrim_Mod:String_Table_File_Format
- `.STRINGS` — null-terminated UTF-8 strings (names)
- `.DLSTRINGS` — uint32 length-prefixed UTF-8 strings (books, notes)
- `.ILSTRINGS` — uint32 length-prefixed UTF-8 strings (dialogue/subtitles)
- Structure: 8-byte header (uint32 count + uint32 data_size) → directory (id + offset pairs) → string data
- **IMPORTANT**: files must be named `_en` (not `_ru`) because PS5 has no way to switch `sLanguage`; the game always loads `_en` files

### UI Translation File
- Path: `Data/Interface/translate_en.txt`
- Encoding: **UTF-16LE with BOM** (0xFF 0xFE)
- Format: `$KEY\tValue\n` (tab-separated, one per line)

### Font System
- Engine: Autodesk Scaleform GFx (Flash/SWF-based UI)
- Font library: `fonts_en.swf` / `fonts_en.gfx` containing vector glyph definitions
- Vanilla fonts: NB Architekt Light, NB Grotesk, Handwritten_Institute — Latin only
- Cyrillic range needed: U+0400–U+04FF (Basic Cyrillic, 94 characters minimum)
- Tool: JPEXS Free Flash Decompiler (FFDec) — Java, works natively on macOS
- Font config: `fontconfig_en.txt` declares `fontlib`, `map` entries, and `validNameChars`/`validBookChars` with full Cyrillic range

### Font Config Format (fontconfig_en.txt)
```
fontlib "fonts_en"
map "$MAIN_Font" = "FontName" Normal
map "$HandwrittenFont" = "FontName" Normal
map "$Controller_Buttons" = "FontName" Normal
...
validNameChars "...АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯёабвгдежзийклмнопрстуфхцчшщъыьэюя..."
validBookChars "...АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯёабвгдежзийклмнопрстуфхцчшщъыьэюя..."
```

### ESM Plugin
- Format: Bethesda plugin (TES4 header record)
- Must have ESM flag set (not ESP)
- Must have Localized Strings flag (0x80) — tells engine to look for .STRINGS files
- HEDR version: 0.96 (Starfield)
- Internal version stamp: 0x022F (559)
- Master dependency: `Starfield.esm`
- Plugin types: Full Master, Medium Master, or Small Master
- For translation mod: **Small Master** is sufficient (minimal record count)

### BA2 Archives
- Format: Bethesda Archive v2/v3
- Type: **General** (not DDS/Texture) for SWF, strings, and text files
- Naming convention: `ModName - Main.ba2` for strings, `ModName - Interface.ba2` for SWF/UI files
- Archives auto-load when named to match the ESM plugin name
- Tool: `bsa-rs` Rust crate (cross-platform, supports Starfield BA2 v2/v3)

### File Mapping (reference mod _ru → production _en)
| Reference (ZoG v0.42)              | Production Output                    | Archive          |
|-------------------------------------|--------------------------------------|------------------|
| `Data/Strings/starfield_ru.*`       | `Data/Strings/starfield_en.*`        | Main.ba2         |
| `Data/Strings/blueprintships-*_ru.*`| `Data/Strings/blueprintships-*_en.*` | Main.ba2         |
| `Data/Strings/constellation_ru.*`   | `Data/Strings/constellation_en.*`    | Main.ba2         |
| `Data/Strings/oldmars_ru.*`         | `Data/Strings/oldmars_en.*`          | Main.ba2         |
| `Data/Interface/fonts_ru.swf`       | `Data/Interface/fonts_en.swf`        | Interface.ba2    |
| `Data/Interface/fontconfig_ru.txt`  | `Data/Interface/fontconfig_en.txt`   | Interface.ba2    |
| `Data/Interface/translate_ru.txt`   | `Data/Interface/translate_en.txt`    | Interface.ba2    |

## Build Toolchain (macOS + GitHub Actions)

### Local Development (macOS M1)
| Task                  | Tool                          | Platform     |
|-----------------------|-------------------------------|--------------|
| Edit font SWF        | JPEXS FFDec (Java)            | Native macOS |
| Edit strings          | Python script / lib-bethesda-strings (Node.js) | Native macOS |
| Edit translate_en.txt | Any text editor (UTF-16LE)    | Native macOS |
| Create ESM            | Spriggit CLI (.NET 8)         | Native macOS |
| Pack BA2              | bsa-rs (Rust)                 | Native macOS |
| Validate              | Custom scripts                | Native macOS |

### CI/CD Pipeline (GitHub Actions)

**Build workflow** — runs on every push and PR:
```yaml
# .github/workflows/build.yml
name: Build Mod
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          lfs: true

      - name: Setup .NET
        uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '8.0.x'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build ESM (Spriggit)
        run: |
          dotnet tool install -g Spriggit.CLI
          spriggit deserialize \
            --InputPath src/plugin \
            --OutputPath dist/StarfieldRussian.esm \
            --GameRelease Starfield

      - name: Pack BA2 (bsa-rs)
        run: |
          cd tools/ba2-packer
          cargo run --release -- \
            --input-strings ../../src/strings \
            --input-interface ../../src/interface \
            --output-dir ../../dist

      - name: Validate
        run: python scripts/validate.py dist/

      - name: Upload Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: starfield-russian-mod
          path: dist/
```

**Release workflow** — creates GitHub Release with downloadable zip on tag push:
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  release:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          lfs: true

      - name: Setup .NET
        uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '8.0.x'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build
        run: |
          dotnet tool install -g Spriggit.CLI
          spriggit deserialize \
            --InputPath src/plugin \
            --OutputPath dist/StarfieldRussian.esm \
            --GameRelease Starfield
          cd tools/ba2-packer
          cargo run --release -- \
            --input-strings ../../src/strings \
            --input-interface ../../src/interface \
            --output-dir ../../dist

      - name: Validate
        run: python scripts/validate.py dist/

      - name: Package
        run: |
          cd dist
          zip -r ../StarfieldRussian-${{ github.ref_name }}.zip .

      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          files: StarfieldRussian-${{ github.ref_name }}.zip
          generate_release_notes: true
```

**Versioning**: semver tags — `v0.1.0`, `v0.2.0`, ... `v1.0.0`.
```bash
# To create a release:
git tag v0.1.0
git push origin v0.1.0
# → GitHub Actions builds, validates, creates Release with zip
```

### Validation Checks
- [ ] ESM has ESM flag set
- [ ] ESM has Localized Strings flag (0x80)
- [ ] ESM HEDR version = 0.96
- [ ] ESM references Starfield.esm as master
- [ ] All 12 string files present in BA2 (4 plugins × 3 types)
- [ ] String files parse correctly (header count matches directory entries)
- [ ] translate_en.txt is valid UTF-16LE with BOM
- [ ] translate_en.txt has $KEY\tValue format on every line
- [ ] fontconfig_en.txt references `fontlib "fonts_en"`
- [ ] fontconfig_en.txt `validNameChars` contains full Cyrillic range
- [ ] fonts_en.swf is valid SWF file
- [ ] BA2 archives are valid Starfield format (General type)
- [ ] Total mod size < 2 GB (Creations upload limit)

## Publishing

### Artifact Distribution
Every tagged release produces a downloadable zip on GitHub Releases:
```
StarfieldRussian-v0.1.0.zip
├── StarfieldRussian.esm
├── StarfieldRussian - Main.ba2
└── StarfieldRussian - Interface.ba2
```
This zip is the single source of truth for all publishing channels.

### Nexus Mods (self-service, browser upload)
1. Download zip from GitHub Releases
2. Upload to https://www.nexusmods.com/starfield/mods/ via browser
3. No Windows required

### Bethesda Creations (PS5 target — requires co-author)
**Requirements for co-author:**
- Windows PC
- Starfield purchased and installed via Steam (~125 GB)
- Creation Kit 2 installed (free on Steam)
- Bethesda.net account

**Co-author upload steps** (documented in `docs/PUBLISH_CREATIONS.md`):
1. Download zip from GitHub Releases
2. Extract contents to `Starfield/Data/` directory
3. Launch Creation Kit 2
4. File → Data → select `StarfieldRussian.esm` → Set as Active File → OK
5. File → Upload to Creations
6. Fill in: name, description, box art, screenshots, tags (Small Master)
7. Submit — mod becomes available on PC, Xbox, and PS5

**For mod updates:**
1. Download new release zip
2. Replace files in `Starfield/Data/`
3. Open in CK2 → Upload update to existing Creation

### GitHub Releases as Distribution Hub
```
Developer (macOS)          GitHub Actions           Consumers
─────────────────          ──────────────           ─────────
git tag v0.1.0  ──push──→  build + validate
                            ├─ ESM (Spriggit)
                            ├─ BA2 (bsa-rs)
                            └─ zip ──→ Release ──→  Co-author → Creations (PS5)
                                           │
                                           ├──────→  Self → Nexus Mods (PC)
                                           │
                                           └──────→  Anyone → manual install (PC)
```

## Reference Materials

### Existing Russian Translation Mods
- **ZoG/Segnet "Unofficial Russian Translation"** — Nexus #1357, Creations (full version with fonts)
  - Version 1.100, 159,000+ translated lines, base game + Shattered Space DLC
  - Reference mod v0.42 extracted and analyzed (see file mapping table above)
- **"Cyrillization for Vanilla Font Collection"** — Nexus #1010 (fonts only, inheritance method)
- **"Russian-Bilingual Localization" by BageDog** — Nexus #1380 (informal "ты" form)

### Tools & Libraries
- JPEXS FFDec: https://github.com/jindrapetrik/jpexs-decompiler
- Mutagen (.NET): https://github.com/Mutagen-Modding/Mutagen
- Spriggit (.NET): https://github.com/Mutagen-Modding/Spriggit
- bsa-rs (Rust): https://github.com/Ryan-rsm-McKenzie/bsa-rs
- starhopper (Python): https://github.com/TkTech/starhopper
- lib-bethesda-strings (Node.js): npm
- xTranslator: https://github.com/MGuffin/xTranslator (Windows only)
- Creation Hub Interface Kit: https://github.com/Creation-Hub/Interface

### Documentation
- String table format: https://en.uesp.net/wiki/Skyrim_Mod:String_Table_File_Format
- ESM/TES4 header: https://en.uesp.net/wiki/Skyrim_Mod:Mod_File_Format/TES4
- Scaleform font system: https://stepmodifications.org/wiki/Guide:UnderstandingFonts
- BA2 format: https://starfieldwiki.net/wiki/Starfield_Mod:Archive2
- Bethesda modding guidelines: https://help.bethesda.net/app/answers/detail/a_id/51731

## Constraints
- Development environment: macOS M1 (no Windows available locally)
- No Starfield PC copy (game owned on PS5 only)
- Publishing to Creations requires a Windows co-author
- GitHub is the single source of truth: code, CI/CD, releases, artifact distribution
- Code and variable names in English, communication in Russian
- Prefer minimal dependencies, no overengineering
- All build steps must be reproducible in CI (no local-only build steps)
- Binary assets (SWF, STRINGS, BA2, ESM) tracked via Git LFS
