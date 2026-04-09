use crate::transliterate::normalize_translate_data;
use anyhow::{bail, Context, Result};
use ba2::fo4::{
    Archive, ArchiveKey, ArchiveOptions, Chunk, CompressionFormat, File, Format, Version,
};
use ba2::prelude::*;
use std::fs;
use std::path::Path;

const MOD_NAME: &str = "StarfieldRussian";

const STRING_EXTENSIONS: &[&str] = &["STRINGS", "DLSTRINGS", "ILSTRINGS"];

const STRING_PREFIXES: &[&str] = &[
    "starfield_en",
    "blueprintships-starfield_en",
    "constellation_en",
    "oldmars_en",
];

const INTERFACE_FILES: &[&str] = &["fonts_en.swf", "fontconfig_en.txt", "translate_en.txt"];

fn make_starfield_options() -> ArchiveOptions {
    ArchiveOptions::builder()
        .format(Format::GNRL)
        .version(Version::v2)
        .compression_format(CompressionFormat::Zip)
        .strings(true)
        .build()
}

fn pack_files(files: Vec<(String, Vec<u8>)>) -> Archive<'static> {
    files
        .into_iter()
        .map(|(path, data)| {
            let chunk = Chunk::from_decompressed(data.into_boxed_slice());
            let file: File = [chunk].into_iter().collect();
            let key: ArchiveKey = path.as_bytes().into();
            (key, file)
        })
        .collect()
}

fn collect_string_files(input_strings: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();

    for prefix in STRING_PREFIXES {
        for ext in STRING_EXTENSIONS {
            let filename = format!("{prefix}.{ext}");
            let filepath = input_strings.join(&filename);
            if filepath.exists() {
                let data = fs::read(&filepath)
                    .with_context(|| format!("Failed to read {}", filepath.display()))?;
                let archive_path = format!("Strings/{filename}");
                files.push((archive_path, data));
            }
        }
    }

    Ok(files)
}

fn collect_interface_files(input_interface: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();

    for filename in INTERFACE_FILES {
        let filepath = input_interface.join(filename);
        if filepath.exists() {
            let mut data = fs::read(&filepath)
                .with_context(|| format!("Failed to read {}", filepath.display()))?;
            // Normalize quoted CSV lines in translate_en.txt before packing
            if *filename == "translate_en.txt" {
                data = normalize_translate_data(&data)
                    .with_context(|| format!("Failed to normalize {}", filepath.display()))?;
            }
            let archive_path = format!("Interface/{filename}");
            files.push((archive_path, data));
        }
    }

    Ok(files)
}

pub fn run(
    input_strings: &Path,
    input_interface: &Path,
    output_dir: &Path,
    credit: Option<&str>,
) -> Result<()> {
    if !input_strings.is_dir() {
        bail!(
            "Input strings directory does not exist: {}",
            input_strings.display()
        );
    }
    if !input_interface.is_dir() {
        bail!(
            "Input interface directory does not exist: {}",
            input_interface.display()
        );
    }

    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    let options = make_starfield_options();

    // Collect all files first, then check
    let string_files = collect_string_files(input_strings)?;
    let interface_files = collect_interface_files(input_interface)?;

    if string_files.is_empty() && interface_files.is_empty() {
        bail!(
            "No artifacts to pack: no string files found in {} and no interface files found in {}",
            input_strings.display(),
            input_interface.display()
        );
    }

    // Pack Main BA2 (strings)
    if string_files.is_empty() {
        eprintln!(
            "Warning: No string files found in {}",
            input_strings.display()
        );
    } else {
        let archive = pack_files(string_files);
        let output_path = output_dir.join(format!("{MOD_NAME} - Main.ba2"));
        let mut output = fs::File::create(&output_path)
            .with_context(|| format!("Failed to create {}", output_path.display()))?;
        archive
            .write(&mut output, &options)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;
        println!("Created: {}", output_path.display());
    }

    // Pack Interface BA2
    if interface_files.is_empty() {
        eprintln!(
            "Warning: No interface files found in {}",
            input_interface.display()
        );
    } else {
        for expected in INTERFACE_FILES {
            let archive_path = format!("Interface/{expected}");
            if !interface_files.iter().any(|(p, _)| p == &archive_path) {
                eprintln!("Warning: Missing interface file: {expected}");
            }
        }
        let archive = pack_files(interface_files);
        let output_path = output_dir.join(format!("{MOD_NAME} - Interface.ba2"));
        let mut output = fs::File::create(&output_path)
            .with_context(|| format!("Failed to create {}", output_path.display()))?;
        archive
            .write(&mut output, &options)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;
        println!("Created: {}", output_path.display());
    }

    // Generate CREDITS.txt if credit is provided
    if let Some(author) = credit {
        let credits_path = output_dir.join("CREDITS.txt");
        let content = format!("Translation by: {author}\n");
        fs::write(&credits_path, content)
            .with_context(|| format!("Failed to write {}", credits_path.display()))?;
        println!("Created: {}", credits_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_make_starfield_options() {
        let opts = make_starfield_options();
        assert_eq!(opts.format(), Format::GNRL);
        assert_eq!(opts.version(), Version::v2);
        assert_eq!(opts.compression_format(), CompressionFormat::Zip);
    }

    #[test]
    fn test_pack_empty_files() {
        let files: Vec<(String, Vec<u8>)> = vec![];
        let archive = pack_files(files);
        assert!(archive.is_empty());
    }

    #[test]
    fn test_pack_single_file() {
        let files = vec![("test.txt".to_string(), b"hello world".to_vec())];
        let archive = pack_files(files);
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_collect_string_files_empty_dir() {
        let dir = TempDir::new().unwrap();
        let files = collect_string_files(dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_string_files_with_files() {
        let dir = TempDir::new().unwrap();

        // Create a test string file
        fs::write(dir.path().join("starfield_en.STRINGS"), b"test data").unwrap();

        let files = collect_string_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "Strings/starfield_en.STRINGS");
        assert_eq!(files[0].1, b"test data");
    }

    #[test]
    fn test_collect_interface_files_empty_dir() {
        let dir = TempDir::new().unwrap();
        let files = collect_interface_files(dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_interface_files_with_files() {
        let dir = TempDir::new().unwrap();

        fs::write(dir.path().join("fontconfig_en.txt"), b"fontlib test").unwrap();

        let files = collect_interface_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "Interface/fontconfig_en.txt");
    }

    #[test]
    fn test_collect_interface_files_normalizes_translate_csv() {
        let dir = TempDir::new().unwrap();

        // Write a translate_en.txt with quoted CSV content (UTF-16LE with BOM)
        let csv_text = "\"$KEY1\tOriginal\",\"Translation\"\n";
        let mut data = vec![0xFF, 0xFE]; // BOM
        for unit in csv_text.encode_utf16() {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        fs::write(dir.path().join("translate_en.txt"), &data).unwrap();

        let files = collect_interface_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "Interface/translate_en.txt");

        // Verify the packed data is normalized (not raw CSV)
        let packed_data = &files[0].1;
        // Decode UTF-16LE (skip BOM)
        let u16s: Vec<u16> = packed_data[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let text = String::from_utf16(&u16s).unwrap();
        assert!(
            text.contains("$KEY1\tTranslation"),
            "Expected normalized format, got: {text}"
        );
        assert!(
            !text.contains('"'),
            "CSV quotes should be stripped, got: {text}"
        );
    }

    #[test]
    fn test_run_nonexistent_input() {
        let result = run(
            Path::new("/nonexistent/strings"),
            Path::new("/nonexistent/interface"),
            Path::new("/tmp/output"),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_creates_ba2_archives() {
        let strings_dir = TempDir::new().unwrap();
        let interface_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create minimal test files
        fs::write(
            strings_dir.path().join("starfield_en.STRINGS"),
            b"\x01\x00\x00\x00\x05\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00test\x00",
        )
        .unwrap();
        fs::write(
            interface_dir.path().join("fontconfig_en.txt"),
            b"fontlib \"fonts_en\"",
        )
        .unwrap();

        run(
            strings_dir.path(),
            interface_dir.path(),
            output_dir.path(),
            None,
        )
        .unwrap();

        assert!(output_dir
            .path()
            .join("StarfieldRussian - Main.ba2")
            .exists());
        assert!(output_dir
            .path()
            .join("StarfieldRussian - Interface.ba2")
            .exists());
    }

    #[test]
    fn test_run_with_credit_creates_credits_txt() {
        let strings_dir = TempDir::new().unwrap();
        let interface_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        fs::write(
            strings_dir.path().join("starfield_en.STRINGS"),
            b"\x01\x00\x00\x00\x05\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00test\x00",
        )
        .unwrap();
        fs::write(
            interface_dir.path().join("fontconfig_en.txt"),
            b"fontlib \"fonts_en\"",
        )
        .unwrap();

        run(
            strings_dir.path(),
            interface_dir.path(),
            output_dir.path(),
            Some("ZoG Forum Team"),
        )
        .unwrap();

        let credits_path = output_dir.path().join("CREDITS.txt");
        assert!(credits_path.exists());
        let content = fs::read_to_string(&credits_path).unwrap();
        assert!(content.contains("ZoG Forum Team"));
    }

    #[test]
    fn test_run_fails_when_no_artifacts() {
        let strings_dir = TempDir::new().unwrap();
        let interface_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Both directories exist but are empty — no string files, no interface files
        let result = run(
            strings_dir.path(),
            interface_dir.path(),
            output_dir.path(),
            None,
        );
        assert!(
            result.is_err(),
            "pack should fail when no artifacts are produced"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("No artifacts"),
            "Error should mention no artifacts, got: {err_msg}"
        );
    }

    #[test]
    fn test_run_succeeds_with_only_string_files() {
        let strings_dir = TempDir::new().unwrap();
        let interface_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        fs::write(
            strings_dir.path().join("starfield_en.STRINGS"),
            b"\x01\x00\x00\x00\x05\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00test\x00",
        )
        .unwrap();

        let result = run(
            strings_dir.path(),
            interface_dir.path(),
            output_dir.path(),
            None,
        );
        assert!(result.is_ok(), "pack should succeed with only string files");
        assert!(output_dir
            .path()
            .join("StarfieldRussian - Main.ba2")
            .exists());
        assert!(!output_dir
            .path()
            .join("StarfieldRussian - Interface.ba2")
            .exists());
    }

    #[test]
    fn test_run_succeeds_with_only_interface_files() {
        let strings_dir = TempDir::new().unwrap();
        let interface_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        fs::write(
            interface_dir.path().join("fontconfig_en.txt"),
            b"fontlib \"fonts_en\"",
        )
        .unwrap();

        let result = run(
            strings_dir.path(),
            interface_dir.path(),
            output_dir.path(),
            None,
        );
        assert!(
            result.is_ok(),
            "pack should succeed with only interface files"
        );
        assert!(output_dir
            .path()
            .join("StarfieldRussian - Interface.ba2")
            .exists());
        assert!(!output_dir
            .path()
            .join("StarfieldRussian - Main.ba2")
            .exists());
    }

    #[test]
    fn test_run_without_credit_no_credits_txt() {
        let strings_dir = TempDir::new().unwrap();
        let interface_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        fs::write(
            strings_dir.path().join("starfield_en.STRINGS"),
            b"\x01\x00\x00\x00\x05\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00test\x00",
        )
        .unwrap();
        fs::write(
            interface_dir.path().join("fontconfig_en.txt"),
            b"fontlib \"fonts_en\"",
        )
        .unwrap();

        run(
            strings_dir.path(),
            interface_dir.path(),
            output_dir.path(),
            None,
        )
        .unwrap();

        assert!(!output_dir.path().join("CREDITS.txt").exists());
    }
}
