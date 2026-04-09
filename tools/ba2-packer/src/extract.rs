use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use crate::string_table::{self, StringEntry, StringTableType};

const STRING_EXTENSIONS: &[&str] = &["STRINGS", "DLSTRINGS", "ILSTRINGS"];

fn collect_string_table_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if STRING_EXTENSIONS
                .iter()
                .any(|&e| e.eq_ignore_ascii_case(ext))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn write_jsonl(entries: &[StringEntry], output_path: &Path) -> Result<()> {
    let mut lines = Vec::with_capacity(entries.len());
    for entry in entries {
        lines.push(serde_json::to_string(entry).context("Failed to serialize entry")?);
    }
    let content = lines.join("\n");
    fs::write(output_path, content)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;
    Ok(())
}

fn extract_file(input_path: &Path, output_dir: &Path) -> Result<()> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .context("Input file has no extension")?;
    let table_type = StringTableType::from_extension(ext)?;

    let data =
        fs::read(input_path).with_context(|| format!("Failed to read {}", input_path.display()))?;
    let entries = string_table::parse_string_table(&data, table_type)?;

    let filename = input_path
        .file_name()
        .context("No filename")?
        .to_str()
        .context("Invalid filename")?;
    let output_path = output_dir.join(format!("{filename}.jsonl"));

    write_jsonl(&entries, &output_path)?;
    println!(
        "Extracted {} entries: {} -> {}",
        entries.len(),
        input_path.display(),
        output_path.display()
    );
    Ok(())
}

pub fn run(input: &Path, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create {}", output_dir.display()))?;

    if input.is_file() {
        extract_file(input, output_dir)
    } else if input.is_dir() {
        let files = collect_string_table_files(input)?;
        if files.is_empty() {
            bail!("No string table files found in {}", input.display());
        }
        for file in &files {
            extract_file(file, output_dir)?;
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
    fn test_write_jsonl_single() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");
        let entries = vec![StringEntry {
            id: 1,
            text: "hello".to_string(),
        }];
        write_jsonl(&entries, &path).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, r#"{"id":1,"text":"hello"}"#);
    }

    #[test]
    fn test_write_jsonl_escaping() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");
        let entries = vec![StringEntry {
            id: 1,
            text: "line1\nline2\t\"quoted\"".to_string(),
        }];
        write_jsonl(&entries, &path).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(r#"\n"#));
        assert!(content.contains(r#"\t"#));
        assert!(content.contains(r#"\""#));
    }

    #[test]
    fn test_extract_file_strings() {
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        let entries = vec![
            StringEntry {
                id: 1,
                text: "hello".to_string(),
            },
            StringEntry {
                id: 2,
                text: "world".to_string(),
            },
        ];
        let binary = write_string_table(&entries, StringTableType::Strings);
        let input_path = input_dir.path().join("starfield_en.STRINGS");
        fs::write(&input_path, &binary).unwrap();

        extract_file(&input_path, output_dir.path()).unwrap();

        let jsonl_path = output_dir.path().join("starfield_en.STRINGS.jsonl");
        assert!(jsonl_path.exists());
        let content = fs::read_to_string(&jsonl_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_run_directory() {
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create two binary files
        let e1 = vec![StringEntry {
            id: 1,
            text: "a".to_string(),
        }];
        let e2 = vec![StringEntry {
            id: 2,
            text: "b".to_string(),
        }];
        fs::write(
            input_dir.path().join("test_en.STRINGS"),
            write_string_table(&e1, StringTableType::Strings),
        )
        .unwrap();
        fs::write(
            input_dir.path().join("test_en.DLSTRINGS"),
            write_string_table(&e2, StringTableType::DlStrings),
        )
        .unwrap();

        run(input_dir.path(), output_dir.path()).unwrap();

        assert!(output_dir.path().join("test_en.STRINGS.jsonl").exists());
        assert!(output_dir.path().join("test_en.DLSTRINGS.jsonl").exists());
    }

    #[test]
    fn test_collect_ignores_other_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.STRINGS"), b"data").unwrap();
        fs::write(dir.path().join("readme.txt"), b"text").unwrap();
        let files = collect_string_table_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }
}
