use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

const FONTCONFIG_SRC: &str = "fontconfig_ru.txt";

const RENAME_MAP: &[(&str, &str)] = &[
    // String tables: starfield
    ("starfield_ru.STRINGS", "starfield_en.STRINGS"),
    ("starfield_ru.DLSTRINGS", "starfield_en.DLSTRINGS"),
    ("starfield_ru.ILSTRINGS", "starfield_en.ILSTRINGS"),
    // String tables: blueprintships-starfield
    (
        "blueprintships-starfield_ru.STRINGS",
        "blueprintships-starfield_en.STRINGS",
    ),
    (
        "blueprintships-starfield_ru.DLSTRINGS",
        "blueprintships-starfield_en.DLSTRINGS",
    ),
    (
        "blueprintships-starfield_ru.ILSTRINGS",
        "blueprintships-starfield_en.ILSTRINGS",
    ),
    // String tables: constellation
    ("constellation_ru.STRINGS", "constellation_en.STRINGS"),
    ("constellation_ru.DLSTRINGS", "constellation_en.DLSTRINGS"),
    ("constellation_ru.ILSTRINGS", "constellation_en.ILSTRINGS"),
    // String tables: oldmars
    ("oldmars_ru.STRINGS", "oldmars_en.STRINGS"),
    ("oldmars_ru.DLSTRINGS", "oldmars_en.DLSTRINGS"),
    ("oldmars_ru.ILSTRINGS", "oldmars_en.ILSTRINGS"),
    // Interface files
    ("fonts_ru.swf", "fonts_en.swf"),
    ("fontconfig_ru.txt", "fontconfig_en.txt"),
    ("translate_ru.txt", "translate_en.txt"),
];

/// Copy or transform a single file from src to dst.
/// For fontconfig, rewrites `"fonts_ru"` → `"fonts_en"` inside the content.
fn copy_file(src_path: &Path, dst_path: &Path, src_name: &str) -> Result<()> {
    if src_name == FONTCONFIG_SRC {
        let content = fs::read_to_string(src_path)
            .with_context(|| format!("Failed to read {}", src_path.display()))?;
        let rewritten = content.replace("\"fonts_ru\"", "\"fonts_en\"");
        fs::write(dst_path, rewritten)
            .with_context(|| format!("Failed to write {}", dst_path.display()))?;
    } else {
        fs::copy(src_path, dst_path).with_context(|| {
            format!(
                "Failed to copy {} -> {}",
                src_path.display(),
                dst_path.display()
            )
        })?;
    }
    Ok(())
}

pub fn run(input_dir: &Path, output_dir: &Path) -> Result<()> {
    if !input_dir.is_dir() {
        bail!("Input directory does not exist: {}", input_dir.display());
    }

    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    let mut found = 0;
    let mut copied = 0;

    for (src_name, dst_name) in RENAME_MAP {
        let src_path = input_dir.join(src_name);

        if !src_path.exists() {
            // Also check in subdirectories (Data/Strings/, Data/Interface/)
            let alt_paths = [
                input_dir.join("Strings").join(src_name),
                input_dir.join("Interface").join(src_name),
                input_dir.join("Data/Strings").join(src_name),
                input_dir.join("Data/Interface").join(src_name),
            ];

            let alt_found = alt_paths.iter().find(|p| p.exists());

            if let Some(alt_path) = alt_found {
                found += 1;
                let dst_path = output_dir.join(dst_name);
                copy_file(alt_path, &dst_path, src_name)?;
                println!("{} -> {}", alt_path.display(), dst_name);
                copied += 1;
            } else {
                eprintln!("Not found: {src_name}");
            }
            continue;
        }

        found += 1;
        let dst_path = output_dir.join(dst_name);
        copy_file(&src_path, &dst_path, src_name)?;
        println!("{src_name} -> {dst_name}");
        copied += 1;
    }

    println!("\nFound: {found}/{}, Copied: {copied}", RENAME_MAP.len());

    if copied == 0 {
        bail!("No files were found to rename. Check input directory structure.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rename_map_completeness() {
        // 12 string files + 3 interface files = 15 total
        assert_eq!(RENAME_MAP.len(), 15);
    }

    #[test]
    fn test_rename_map_consistency() {
        for (src, dst) in RENAME_MAP {
            assert!(src.contains("_ru"), "Source {src} should contain _ru");
            assert!(dst.contains("_en"), "Dest {dst} should contain _en");
            // Same extension
            let src_ext = src.rsplit('.').next().unwrap();
            let dst_ext = dst.rsplit('.').next().unwrap();
            assert_eq!(src_ext, dst_ext, "Extensions must match: {src} vs {dst}");
        }
    }

    #[test]
    fn test_run_nonexistent_input() {
        let result = run(Path::new("/nonexistent"), Path::new("/tmp/out"));
        assert!(result.is_err());
    }

    #[test]
    fn test_run_empty_input() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();
        let result = run(input.path(), output.path());
        // Should fail because no files found
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_files() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        // Create some _ru files
        fs::write(input.path().join("starfield_ru.STRINGS"), b"test").unwrap();
        fs::write(input.path().join("fonts_ru.swf"), b"FWS\x09").unwrap();

        let result = run(input.path(), output.path());
        assert!(result.is_ok());

        // Check renamed files exist
        assert!(output.path().join("starfield_en.STRINGS").exists());
        assert!(output.path().join("fonts_en.swf").exists());
    }

    #[test]
    fn test_run_with_subdirectories() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        // Create files in Strings/ subdirectory
        let strings_dir = input.path().join("Strings");
        fs::create_dir(&strings_dir).unwrap();
        fs::write(strings_dir.join("starfield_ru.STRINGS"), b"test").unwrap();

        let result = run(input.path(), output.path());
        assert!(result.is_ok());

        assert!(output.path().join("starfield_en.STRINGS").exists());
    }

    #[test]
    fn test_fontconfig_content_rewritten() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        // Create fontconfig_ru.txt with fontlib "fonts_ru"
        fs::write(
            input.path().join("fontconfig_ru.txt"),
            b"fontlib \"fonts_ru\"\nmap \"$MAIN_Font\" = \"RF_35_M\" Normal",
        )
        .unwrap();

        let result = run(input.path(), output.path());
        assert!(result.is_ok());

        let content = fs::read_to_string(output.path().join("fontconfig_en.txt")).unwrap();
        assert!(
            content.contains("fontlib \"fonts_en\""),
            "fontconfig should reference fonts_en, got: {content}"
        );
        assert!(
            !content.contains("fontlib \"fonts_ru\""),
            "fontconfig should not reference fonts_ru"
        );
        // Other content preserved
        assert!(content.contains("map \"$MAIN_Font\" = \"RF_35_M\" Normal"));
    }
}
