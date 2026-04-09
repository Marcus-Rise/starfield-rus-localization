use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use crate::string_table::{self, StringEntry, StringTableType};

fn read_jsonl(input_path: &Path) -> Result<Vec<StringEntry>> {
    let content = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read {}", input_path.display()))?;

    let mut entries = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let entry: StringEntry = serde_json::from_str(line).with_context(|| {
            format!("Invalid JSON on line {} of {}", i + 1, input_path.display())
        })?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Extract the original binary filename and table type from a `.jsonl` path.
///
/// E.g. `starfield_en.STRINGS.jsonl` -> `("starfield_en.STRINGS", Strings)`
fn output_filename_from_jsonl(jsonl_path: &Path) -> Result<(String, StringTableType)> {
    let filename = jsonl_path
        .file_name()
        .and_then(|f| f.to_str())
        .context("Invalid jsonl filename")?;

    let stem = filename
        .strip_suffix(".jsonl")
        .context("File does not end with .jsonl")?;

    let ext = stem.rsplit('.').next().context("No extension in stem")?;

    let table_type = StringTableType::from_extension(ext)?;
    Ok((stem.to_string(), table_type))
}

fn repack_file(input_path: &Path, output_dir: &Path) -> Result<()> {
    let (output_name, table_type) = output_filename_from_jsonl(input_path)?;
    let entries = read_jsonl(input_path)?;
    let binary = string_table::write_string_table(&entries, table_type);

    let output_path = output_dir.join(&output_name);
    fs::write(&output_path, &binary)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    println!(
        "Repacked {} entries: {} -> {}",
        entries.len(),
        input_path.display(),
        output_path.display()
    );
    Ok(())
}

fn collect_jsonl_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            // Verify it has a valid string table extension before .jsonl
            if output_filename_from_jsonl(&path).is_ok() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

pub fn run(input: &Path, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create {}", output_dir.display()))?;

    if input.is_file() {
        repack_file(input, output_dir)
    } else if input.is_dir() {
        let files = collect_jsonl_files(input)?;
        if files.is_empty() {
            bail!("No valid .jsonl files found in {}", input.display());
        }
        for file in &files {
            repack_file(file, output_dir)?;
        }
        Ok(())
    } else {
        bail!("Input path does not exist: {}", input.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_table::write_string_table;
    use tempfile::TempDir;

    #[test]
    fn test_read_jsonl_single() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");
        fs::write(&path, r#"{"id":1,"text":"hello"}"#).unwrap();
        let entries = read_jsonl(&path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 1);
        assert_eq!(entries[0].text, "hello");
    }

    #[test]
    fn test_read_jsonl_multiple() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");
        fs::write(
            &path,
            "{\"id\":1,\"text\":\"a\"}\n{\"id\":2,\"text\":\"b\"}",
        )
        .unwrap();
        let entries = read_jsonl(&path).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_read_jsonl_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");
        fs::write(&path, "").unwrap();
        let entries = read_jsonl(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_read_jsonl_malformed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");
        fs::write(&path, "not json").unwrap();
        assert!(read_jsonl(&path).is_err());
    }

    #[test]
    fn test_output_filename_strings() {
        let path = Path::new("starfield_en.STRINGS.jsonl");
        let (name, table_type) = output_filename_from_jsonl(path).unwrap();
        assert_eq!(name, "starfield_en.STRINGS");
        assert_eq!(table_type, StringTableType::Strings);
    }

    #[test]
    fn test_output_filename_dlstrings() {
        let path = Path::new("starfield_en.DLSTRINGS.jsonl");
        let (name, table_type) = output_filename_from_jsonl(path).unwrap();
        assert_eq!(name, "starfield_en.DLSTRINGS");
        assert_eq!(table_type, StringTableType::DlStrings);
    }

    #[test]
    fn test_output_filename_bad_extension() {
        let path = Path::new("somefile.jsonl");
        assert!(output_filename_from_jsonl(path).is_err());
    }

    #[test]
    fn test_repack_file_strings() {
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        let jsonl = "{\"id\":1,\"text\":\"hello\"}\n{\"id\":2,\"text\":\"world\"}";
        let jsonl_path = input_dir.path().join("test_en.STRINGS.jsonl");
        fs::write(&jsonl_path, jsonl).unwrap();

        repack_file(&jsonl_path, output_dir.path()).unwrap();

        let output_path = output_dir.path().join("test_en.STRINGS");
        assert!(output_path.exists());

        // Verify the binary is valid
        let data = fs::read(&output_path).unwrap();
        let entries =
            crate::string_table::parse_string_table(&data, StringTableType::Strings).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "hello");
    }

    #[test]
    fn test_extract_repack_round_trip() {
        let original_dir = TempDir::new().unwrap();
        let jsonl_dir = TempDir::new().unwrap();
        let repacked_dir = TempDir::new().unwrap();

        // Create original binary
        let entries = vec![
            StringEntry {
                id: 10,
                text: "Привет".to_string(),
            },
            StringEntry {
                id: 20,
                text: "мир".to_string(),
            },
        ];
        let binary = write_string_table(&entries, StringTableType::Strings);
        let original_path = original_dir.path().join("test_en.STRINGS");
        fs::write(&original_path, &binary).unwrap();

        // Extract
        crate::extract::run(&original_path, jsonl_dir.path()).unwrap();

        // Repack
        let jsonl_path = jsonl_dir.path().join("test_en.STRINGS.jsonl");
        crate::repack::run(&jsonl_path, repacked_dir.path()).unwrap();

        // Compare
        let repacked = fs::read(repacked_dir.path().join("test_en.STRINGS")).unwrap();
        assert_eq!(binary, repacked, "Round-trip must produce identical bytes");
    }
}
