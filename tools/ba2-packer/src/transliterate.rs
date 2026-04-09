use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use crate::string_table::{self, StringEntry, StringTableType};

/// Map a single Cyrillic character to its Latin transliteration.
/// Returns `None` for non-Cyrillic characters.
/// Returns `Some("")` for silent letters (Ъ, Ь).
fn transliterate_char(c: char) -> Option<&'static str> {
    match c {
        // Uppercase
        'А' => Some("A"),
        'Б' => Some("B"),
        'В' => Some("V"),
        'Г' => Some("G"),
        'Д' => Some("D"),
        'Е' | 'Э' => Some("E"),
        'Ё' => Some("Yo"),
        'Ж' => Some("Zh"),
        'З' => Some("Z"),
        'И' => Some("I"),
        'Й' => Some("J"),
        'К' => Some("K"),
        'Л' => Some("L"),
        'М' => Some("M"),
        'Н' => Some("N"),
        'О' => Some("O"),
        'П' => Some("P"),
        'Р' => Some("R"),
        'С' => Some("S"),
        'Т' => Some("T"),
        'У' => Some("U"),
        'Ф' => Some("F"),
        'Х' => Some("Kh"),
        'Ц' => Some("Ts"),
        'Ч' => Some("Ch"),
        'Ш' => Some("Sh"),
        'Щ' => Some("Sch"),
        'Ъ' | 'Ь' | 'ъ' | 'ь' => Some(""),
        'Ы' => Some("Y"),
        'Ю' => Some("Yu"),
        'Я' => Some("Ya"),
        // Lowercase
        'а' => Some("a"),
        'б' => Some("b"),
        'в' => Some("v"),
        'г' => Some("g"),
        'д' => Some("d"),
        'е' | 'э' => Some("e"),
        'ё' => Some("yo"),
        'ж' => Some("zh"),
        'з' => Some("z"),
        'и' => Some("i"),
        'й' => Some("j"),
        'к' => Some("k"),
        'л' => Some("l"),
        'м' => Some("m"),
        'н' => Some("n"),
        'о' => Some("o"),
        'п' => Some("p"),
        'р' => Some("r"),
        'с' => Some("s"),
        'т' => Some("t"),
        'у' => Some("u"),
        'ф' => Some("f"),
        'х' => Some("kh"),
        'ц' => Some("ts"),
        'ч' => Some("ch"),
        'ш' => Some("sh"),
        'щ' => Some("sch"),
        'ы' => Some("y"),
        'ю' => Some("yu"),
        'я' => Some("ya"),
        _ => None,
    }
}

/// Transliterate a string: Cyrillic characters become Latin, everything else preserved.
fn transliterate(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match transliterate_char(c) {
            Some(latin) => result.push_str(latin),
            None => result.push(c),
        }
    }
    result
}

/// Parse a binary string table, transliterate all text entries, and serialize back.
fn transliterate_string_table(data: &[u8], table_type: StringTableType) -> Result<Vec<u8>> {
    let entries = string_table::parse_string_table(data, table_type)?;
    let transliterated: Vec<StringEntry> = entries
        .into_iter()
        .map(|e| StringEntry {
            id: e.id,
            text: transliterate(&e.text),
        })
        .collect();
    Ok(string_table::write_string_table(
        &transliterated,
        table_type,
    ))
}

/// Normalize a quoted CSV line to `$KEY\tValue` format.
///
/// Handles two CSV variants:
/// - `"$KEY\tOriginal","Translation"` → `$KEY\tTranslation`
/// - `"$KEY","Value"` → `$KEY\tValue`
///
/// Lines already in standard `$KEY\tValue` format pass through unchanged.
pub(crate) fn normalize_csv_line(line: &str) -> String {
    // Detect quoted CSV: starts with "$ and contains ","
    if !line.starts_with("\"$") {
        return line.to_string();
    }
    let Some(sep_pos) = line.find("\",\"") else {
        return line.to_string();
    };

    // Field 1: everything between leading " and the ","
    let field1 = &line[1..sep_pos];
    // Field 2: everything between "," and trailing "
    let field2_start = sep_pos + 3; // skip ","
    let field2 = if line.ends_with('"') {
        &line[field2_start..line.len() - 1]
    } else {
        &line[field2_start..]
    };

    // Extract key from field1: everything before the first tab (if any)
    let key = if let Some(tab_pos) = field1.find('\t') {
        &field1[..tab_pos]
    } else {
        field1
    };

    format!("{key}\t{field2}")
}

/// Process a `translate_en.txt` file (UTF-16LE with BOM).
/// Lines have `$KEY\tValue` format — keys are preserved, values are transliterated.
fn transliterate_translate_file(data: &[u8]) -> Result<Vec<u8>> {
    // Strip UTF-16LE BOM if present (FF FE)
    let raw = if data.starts_with(&[0xFF, 0xFE]) {
        &data[2..]
    } else {
        data
    };

    // Decode UTF-16LE
    if raw.len() % 2 != 0 {
        bail!("translate file has odd byte count, not valid UTF-16LE");
    }
    let u16_units: Vec<u16> = raw
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    let text = String::from_utf16(&u16_units).context("Invalid UTF-16LE in translate file")?;

    // Process lines: $KEY\tValue — keep key, transliterate value
    // Normalize quoted CSV lines first (e.g., "$KEY\tOrig","Translation" → $KEY\tTranslation)
    let mut output_lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let normalized = normalize_csv_line(line);
        if let Some(tab_pos) = normalized.find('\t') {
            let key = &normalized[..tab_pos];
            let value = &normalized[tab_pos + 1..];
            output_lines.push(format!("{}\t{}", key, transliterate(value)));
        } else {
            // No tab — pass through (empty lines, comments, etc.)
            output_lines.push(normalized);
        }
    }

    // Re-encode to UTF-16LE with BOM
    let joined = output_lines.join("\n");
    // Add trailing newline if original had one
    let final_text = if text.ends_with('\n') {
        format!("{joined}\n")
    } else {
        joined
    };

    let mut result: Vec<u8> = vec![0xFF, 0xFE]; // BOM
    for unit in final_text.encode_utf16() {
        result.extend_from_slice(&unit.to_le_bytes());
    }

    Ok(result)
}

/// Write a CREDITS.txt file with translation attribution.
fn write_credits(output_dir: &Path, credit: &str) -> Result<()> {
    let content = format!(
        "Translation credit: {credit}\n\
         \n\
         This transliteration was created from a third-party translation.\n\
         All rights to the original translation belong to the original author(s).\n"
    );
    let path = output_dir.join("CREDITS.txt");
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Find string table files in a directory, checking subdirectories too.
/// When the same basename appears in multiple search directories, only the first match is kept.
fn find_string_tables(input_dir: &Path) -> Vec<(std::path::PathBuf, StringTableType)> {
    let extensions = ["STRINGS", "DLSTRINGS", "ILSTRINGS"];
    let search_dirs = [
        input_dir.to_path_buf(),
        input_dir.join("Strings"),
        input_dir.join("Data").join("Strings"),
    ];

    let mut seen_names = std::collections::HashSet::new();
    let mut found = Vec::new();
    for dir in &search_dirs {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
                        if let Ok(table_type) = StringTableType::from_extension(ext) {
                            let name = path.file_name().unwrap().to_os_string();
                            if seen_names.insert(name) {
                                found.push((path, table_type));
                            }
                        }
                    }
                }
            }
        }
    }
    found
}

/// Find `translate_en.txt` in a directory, checking subdirectories too.
fn find_translate_file(input_dir: &Path) -> Option<std::path::PathBuf> {
    let candidates = [
        input_dir.join("translate_en.txt"),
        input_dir.join("Interface").join("translate_en.txt"),
        input_dir
            .join("Data")
            .join("Interface")
            .join("translate_en.txt"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// Run the transliterate subcommand.
pub fn run(input_dir: &Path, output_dir: &Path, credit: Option<&str>) -> Result<()> {
    if !input_dir.is_dir() {
        bail!("Input directory does not exist: {}", input_dir.display());
    }

    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    let mut processed = 0u32;

    // Process string table files
    let string_tables = find_string_tables(input_dir);
    for (path, table_type) in &string_tables {
        let data = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
        let result = transliterate_string_table(&data, *table_type)
            .with_context(|| format!("Failed to transliterate {}", path.display()))?;

        let file_name = path.file_name().unwrap();
        let dst = output_dir.join(file_name);
        fs::write(&dst, result).with_context(|| format!("Failed to write {}", dst.display()))?;

        println!(
            "Transliterated: {} -> {}",
            path.display(),
            file_name.to_string_lossy()
        );
        processed += 1;
    }

    // Process translate_en.txt
    if let Some(translate_path) = find_translate_file(input_dir) {
        let data = fs::read(&translate_path)
            .with_context(|| format!("Failed to read {}", translate_path.display()))?;
        let result = transliterate_translate_file(&data)
            .with_context(|| format!("Failed to transliterate {}", translate_path.display()))?;

        let dst = output_dir.join("translate_en.txt");
        fs::write(&dst, result).with_context(|| format!("Failed to write {}", dst.display()))?;

        println!(
            "Transliterated: {} -> translate_en.txt",
            translate_path.display()
        );
        processed += 1;
    }

    // Handle credits
    if let Some(credit_name) = credit {
        write_credits(output_dir, credit_name)?;
        println!(
            "\nWARNING: This transliteration uses a third-party translation by: {credit_name}"
        );
        println!("CREDITS.txt has been created in the output directory.");
    }

    println!("\nProcessed: {processed} file(s)");

    if processed == 0 {
        bail!("No files were found to transliterate. Check input directory structure.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    // --- transliterate_char ---

    #[test]
    fn test_transliterate_char_uppercase() {
        assert_eq!(transliterate_char('А'), Some("A"));
        assert_eq!(transliterate_char('Б'), Some("B"));
        assert_eq!(transliterate_char('В'), Some("V"));
        assert_eq!(transliterate_char('Г'), Some("G"));
        assert_eq!(transliterate_char('Д'), Some("D"));
        assert_eq!(transliterate_char('Е'), Some("E"));
        assert_eq!(transliterate_char('Ж'), Some("Zh"));
        assert_eq!(transliterate_char('З'), Some("Z"));
        assert_eq!(transliterate_char('И'), Some("I"));
        assert_eq!(transliterate_char('Й'), Some("J"));
        assert_eq!(transliterate_char('К'), Some("K"));
        assert_eq!(transliterate_char('Л'), Some("L"));
        assert_eq!(transliterate_char('М'), Some("M"));
        assert_eq!(transliterate_char('Н'), Some("N"));
        assert_eq!(transliterate_char('О'), Some("O"));
        assert_eq!(transliterate_char('П'), Some("P"));
        assert_eq!(transliterate_char('Р'), Some("R"));
        assert_eq!(transliterate_char('С'), Some("S"));
        assert_eq!(transliterate_char('Т'), Some("T"));
        assert_eq!(transliterate_char('У'), Some("U"));
        assert_eq!(transliterate_char('Ф'), Some("F"));
        assert_eq!(transliterate_char('Х'), Some("Kh"));
        assert_eq!(transliterate_char('Ц'), Some("Ts"));
        assert_eq!(transliterate_char('Ч'), Some("Ch"));
        assert_eq!(transliterate_char('Ш'), Some("Sh"));
        assert_eq!(transliterate_char('Щ'), Some("Sch"));
        assert_eq!(transliterate_char('Ы'), Some("Y"));
        assert_eq!(transliterate_char('Э'), Some("E"));
        assert_eq!(transliterate_char('Ю'), Some("Yu"));
        assert_eq!(transliterate_char('Я'), Some("Ya"));
    }

    #[test]
    fn test_transliterate_char_lowercase() {
        assert_eq!(transliterate_char('а'), Some("a"));
        assert_eq!(transliterate_char('б'), Some("b"));
        assert_eq!(transliterate_char('в'), Some("v"));
        assert_eq!(transliterate_char('ж'), Some("zh"));
        assert_eq!(transliterate_char('х'), Some("kh"));
        assert_eq!(transliterate_char('ц'), Some("ts"));
        assert_eq!(transliterate_char('ч'), Some("ch"));
        assert_eq!(transliterate_char('ш'), Some("sh"));
        assert_eq!(transliterate_char('щ'), Some("sch"));
        assert_eq!(transliterate_char('ю'), Some("yu"));
        assert_eq!(transliterate_char('я'), Some("ya"));
    }

    #[test]
    fn test_transliterate_char_yo() {
        assert_eq!(transliterate_char('Ё'), Some("Yo"));
        assert_eq!(transliterate_char('ё'), Some("yo"));
    }

    #[test]
    fn test_transliterate_char_silent() {
        assert_eq!(transliterate_char('Ъ'), Some(""));
        assert_eq!(transliterate_char('Ь'), Some(""));
        assert_eq!(transliterate_char('ъ'), Some(""));
        assert_eq!(transliterate_char('ь'), Some(""));
    }

    #[test]
    fn test_transliterate_char_non_cyrillic() {
        assert_eq!(transliterate_char('A'), None);
        assert_eq!(transliterate_char('z'), None);
        assert_eq!(transliterate_char('1'), None);
        assert_eq!(transliterate_char(' '), None);
        assert_eq!(transliterate_char('!'), None);
    }

    // --- transliterate (full string) ---

    #[test]
    fn test_transliterate_simple() {
        assert_eq!(transliterate("Привет"), "Privet");
    }

    #[test]
    fn test_transliterate_mixed() {
        assert_eq!(transliterate("Hello Мир!"), "Hello Mir!");
    }

    #[test]
    fn test_transliterate_empty() {
        assert_eq!(transliterate(""), "");
    }

    #[test]
    fn test_transliterate_no_cyrillic() {
        assert_eq!(transliterate("Hello World 123!"), "Hello World 123!");
    }

    #[test]
    fn test_transliterate_all_cyrillic() {
        assert_eq!(transliterate("АБВГД"), "ABVGD");
        assert_eq!(transliterate("абвгд"), "abvgd");
    }

    #[test]
    fn test_transliterate_complex_chars() {
        assert_eq!(transliterate("Щука"), "Schuka");
        assert_eq!(transliterate("Ёж"), "Yozh");
        assert_eq!(transliterate("Объект"), "Ob'ekt".replace("'", ""));
        // Ъ is silent, so Объект -> Ob + "" + ekt = Obekt
        assert_eq!(transliterate("Объект"), "Obekt");
    }

    #[test]
    fn test_transliterate_with_numbers_and_punctuation() {
        assert_eq!(transliterate("Уровень 5: Победа!"), "Uroven 5: Pobeda!");
    }

    // --- transliterate_string_table ---

    fn build_strings_binary(entries: &[(u32, &str)]) -> Vec<u8> {
        let mut data_buf = Vec::new();
        let mut offsets = Vec::new();
        for (_, text) in entries {
            offsets.push(data_buf.len());
            data_buf.extend_from_slice(text.as_bytes());
            data_buf.push(0);
        }
        let count = entries.len() as u32;
        let data_size = data_buf.len() as u32;
        let mut buf = Vec::new();
        buf.extend_from_slice(&count.to_le_bytes());
        buf.extend_from_slice(&data_size.to_le_bytes());
        for (i, (id, _)) in entries.iter().enumerate() {
            buf.extend_from_slice(&id.to_le_bytes());
            buf.extend_from_slice(&(offsets[i] as u32).to_le_bytes());
        }
        buf.extend_from_slice(&data_buf);
        buf
    }

    fn build_dlstrings_binary(entries: &[(u32, &str)]) -> Vec<u8> {
        let mut data_buf = Vec::new();
        let mut offsets = Vec::new();
        for (_, text) in entries {
            offsets.push(data_buf.len());
            let bytes = text.as_bytes();
            data_buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            data_buf.extend_from_slice(bytes);
        }
        let count = entries.len() as u32;
        let data_size = data_buf.len() as u32;
        let mut buf = Vec::new();
        buf.extend_from_slice(&count.to_le_bytes());
        buf.extend_from_slice(&data_size.to_le_bytes());
        for (i, (id, _)) in entries.iter().enumerate() {
            buf.extend_from_slice(&id.to_le_bytes());
            buf.extend_from_slice(&(offsets[i] as u32).to_le_bytes());
        }
        buf.extend_from_slice(&data_buf);
        buf
    }

    #[test]
    fn test_transliterate_string_table_strings() {
        let data = build_strings_binary(&[(1, "Привет"), (2, "Мир")]);
        let result = transliterate_string_table(&data, StringTableType::Strings).unwrap();
        let entries = string_table::parse_string_table(&result, StringTableType::Strings).unwrap();
        assert_eq!(entries[0].text, "Privet");
        assert_eq!(entries[1].text, "Mir");
    }

    #[test]
    fn test_transliterate_string_table_dlstrings() {
        let data = build_dlstrings_binary(&[(1, "Текст книги")]);
        let result = transliterate_string_table(&data, StringTableType::DlStrings).unwrap();
        let entries =
            string_table::parse_string_table(&result, StringTableType::DlStrings).unwrap();
        assert_eq!(entries[0].text, "Tekst knigi");
    }

    #[test]
    fn test_transliterate_string_table_ilstrings() {
        let data = build_dlstrings_binary(&[(1, "Диалог")]);
        let result = transliterate_string_table(&data, StringTableType::IlStrings).unwrap();
        let entries =
            string_table::parse_string_table(&result, StringTableType::IlStrings).unwrap();
        assert_eq!(entries[0].text, "Dialog");
    }

    #[test]
    fn test_transliterate_string_table_preserves_ids() {
        let data = build_strings_binary(&[(42, "Тест"), (99, "Значение")]);
        let result = transliterate_string_table(&data, StringTableType::Strings).unwrap();
        let entries = string_table::parse_string_table(&result, StringTableType::Strings).unwrap();
        assert_eq!(entries[0].id, 42);
        assert_eq!(entries[1].id, 99);
    }

    #[test]
    fn test_transliterate_string_table_mixed_content() {
        let data = build_strings_binary(&[(1, "Hello Мир!")]);
        let result = transliterate_string_table(&data, StringTableType::Strings).unwrap();
        let entries = string_table::parse_string_table(&result, StringTableType::Strings).unwrap();
        assert_eq!(entries[0].text, "Hello Mir!");
    }

    // --- transliterate_translate_file ---

    fn make_utf16le_with_bom(text: &str) -> Vec<u8> {
        let mut result: Vec<u8> = vec![0xFF, 0xFE]; // BOM
        for unit in text.encode_utf16() {
            result.extend_from_slice(&unit.to_le_bytes());
        }
        result
    }

    #[test]
    fn test_transliterate_translate_file_basic() {
        let input = make_utf16le_with_bom("$KEY1\tЗначение\n$KEY2\tТекст");
        let result = transliterate_translate_file(&input).unwrap();
        // Decode output
        let output = &result[2..]; // skip BOM
        let u16s: Vec<u16> = output
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let text = String::from_utf16(&u16s).unwrap();
        assert!(text.contains("$KEY1\tZnachenie"));
        assert!(text.contains("$KEY2\tTekst"));
    }

    #[test]
    fn test_transliterate_translate_file_preserves_keys() {
        let input = make_utf16le_with_bom("$Привет\tМир");
        let result = transliterate_translate_file(&input).unwrap();
        let output = &result[2..];
        let u16s: Vec<u16> = output
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let text = String::from_utf16(&u16s).unwrap();
        // Key is preserved as-is (including Cyrillic in key)
        assert!(text.contains("$Привет\tMir"));
    }

    #[test]
    fn test_transliterate_translate_file_has_bom() {
        let input = make_utf16le_with_bom("$K\tV");
        let result = transliterate_translate_file(&input).unwrap();
        assert_eq!(&result[..2], &[0xFF, 0xFE]);
    }

    #[test]
    fn test_transliterate_translate_file_odd_bytes() {
        let result = transliterate_translate_file(&[0xFF, 0xFE, 0x00]);
        assert!(result.is_err());
    }

    // --- write_credits ---

    #[test]
    fn test_write_credits() {
        let dir = TempDir::new().unwrap();
        write_credits(dir.path(), "TestAuthor").unwrap();
        let content = fs::read_to_string(dir.path().join("CREDITS.txt")).unwrap();
        assert!(content.contains("TestAuthor"));
        assert!(content.contains("Translation credit"));
    }

    // --- run ---

    #[test]
    fn test_run_nonexistent_input() {
        let result = run(Path::new("/nonexistent"), Path::new("/tmp/out"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_empty_input() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();
        let result = run(input.path(), output.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_string_files() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        let data = build_strings_binary(&[(1, "Привет")]);
        fs::write(input.path().join("starfield_en.STRINGS"), &data).unwrap();

        let result = run(input.path(), output.path(), None);
        assert!(result.is_ok());

        let out_data = fs::read(output.path().join("starfield_en.STRINGS")).unwrap();
        let entries =
            string_table::parse_string_table(&out_data, StringTableType::Strings).unwrap();
        assert_eq!(entries[0].text, "Privet");
    }

    #[test]
    fn test_run_with_credit() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        let data = build_strings_binary(&[(1, "Тест")]);
        fs::write(input.path().join("test_en.STRINGS"), &data).unwrap();

        let result = run(input.path(), output.path(), Some("ZoG"));
        assert!(result.is_ok());
        assert!(output.path().join("CREDITS.txt").exists());
    }

    #[test]
    fn test_run_without_credit() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        let data = build_strings_binary(&[(1, "Тест")]);
        fs::write(input.path().join("test_en.STRINGS"), &data).unwrap();

        let result = run(input.path(), output.path(), None);
        assert!(result.is_ok());
        assert!(!output.path().join("CREDITS.txt").exists());
    }

    #[test]
    fn test_run_with_subdirectory() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        let strings_dir = input.path().join("Strings");
        fs::create_dir(&strings_dir).unwrap();
        let data = build_strings_binary(&[(1, "Мир")]);
        fs::write(strings_dir.join("starfield_en.STRINGS"), &data).unwrap();

        let result = run(input.path(), output.path(), None);
        assert!(result.is_ok());
        assert!(output.path().join("starfield_en.STRINGS").exists());
    }

    #[test]
    fn test_run_with_translate_file() {
        let input = TempDir::new().unwrap();
        let output = TempDir::new().unwrap();

        let translate_data = make_utf16le_with_bom("$KEY1\tЗначение");
        fs::write(input.path().join("translate_en.txt"), &translate_data).unwrap();

        let result = run(input.path(), output.path(), None);
        assert!(result.is_ok());
        assert!(output.path().join("translate_en.txt").exists());
    }

    #[test]
    fn test_find_string_tables_deduplicates_by_basename() {
        let input = TempDir::new().unwrap();

        // Place same-named file in root and Strings/ subdirectory
        let root_data = build_strings_binary(&[(1, "root")]);
        fs::write(input.path().join("starfield_en.STRINGS"), &root_data).unwrap();

        let sub_dir = input.path().join("Strings");
        fs::create_dir(&sub_dir).unwrap();
        let sub_data = build_strings_binary(&[(1, "subdir")]);
        fs::write(sub_dir.join("starfield_en.STRINGS"), &sub_data).unwrap();

        let found = find_string_tables(input.path());
        // Should find only one entry (root takes priority)
        let matching: Vec<_> = found
            .iter()
            .filter(|(p, _)| p.file_name().unwrap() == "starfield_en.STRINGS")
            .collect();
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].0.parent().unwrap(), input.path());
    }

    // --- performance guard tests ---

    #[test]
    fn test_transliterate_perf_pure_string() {
        let sentence = "Космическая станция на орбите планеты ";
        let large_input: String = sentence.repeat(14_500); // ~1 MB of Cyrillic UTF-8

        let start = Instant::now();
        let result = transliterate(&large_input);
        let elapsed = start.elapsed();

        assert!(!result.is_empty());
        assert_ne!(result, large_input);

        let threshold = Duration::from_secs(5);
        assert!(
            elapsed < threshold,
            "transliterate() took {elapsed:?} for ~1MB input, exceeds {threshold:?} threshold"
        );
    }

    #[test]
    fn test_transliterate_perf_string_table() {
        let phrase = "Привет мир! ";
        let entry_text = phrase.repeat(8); // ~96 chars per entry
        let entries: Vec<(u32, &str)> = (0..10_000)
            .map(|i| (i as u32, entry_text.as_str()))
            .collect();
        let data = build_strings_binary(&entries);

        let start = Instant::now();
        let result = transliterate_string_table(&data, StringTableType::Strings);
        let elapsed = start.elapsed();

        assert!(
            result.is_ok(),
            "transliterate_string_table failed: {:?}",
            result.err()
        );

        let threshold = Duration::from_secs(5);
        assert!(
            elapsed < threshold,
            "transliterate_string_table() took {elapsed:?} for 10,000 entries, exceeds {threshold:?} threshold"
        );
    }

    // --- normalize_csv_line ---

    #[test]
    fn test_normalize_csv_line_standard_passthrough() {
        assert_eq!(normalize_csv_line("$KEY\tValue"), "$KEY\tValue");
    }

    #[test]
    fn test_normalize_csv_line_quoted_csv_with_tab_in_field1() {
        // Field 1 has key+tab+original, field 2 has translation
        let input = "\"$SOL DATE: MAY 7\t 2330\",\"SOLNECHNAYA DATA: 7 MAYA 2330 GODA\"";
        let expected = "$SOL DATE: MAY 7\tSOLNECHNAYA DATA: 7 MAYA 2330 GODA";
        assert_eq!(normalize_csv_line(input), expected);
    }

    #[test]
    fn test_normalize_csv_line_no_tab_in_first_field() {
        // Field 1 is just the key, field 2 is the value
        let input = "\"$MY_KEY\",\"Some translation\"";
        let expected = "$MY_KEY\tSome translation";
        assert_eq!(normalize_csv_line(input), expected);
    }

    #[test]
    fn test_normalize_csv_line_empty() {
        assert_eq!(normalize_csv_line(""), "");
    }

    #[test]
    fn test_normalize_csv_line_not_csv() {
        // Line without "$" prefix inside quotes — passthrough
        assert_eq!(normalize_csv_line("just some text"), "just some text");
    }

    #[test]
    fn test_normalize_csv_line_quoted_but_no_second_field() {
        // Starts with "$ but no "," separator — passthrough
        let input = "\"$KEY_ONLY\"";
        assert_eq!(normalize_csv_line(input), "\"$KEY_ONLY\"");
    }

    #[test]
    fn test_transliterate_translate_file_quoted_csv() {
        // Full pipeline: CSV input with Cyrillic values should be normalized and transliterated
        let input_text = "\"$KEY1\tОригинал\",\"Значение\"\n\"$KEY2\",\"Текст\"";
        let input = make_utf16le_with_bom(input_text);
        let result = transliterate_translate_file(&input).unwrap();
        let output = &result[2..]; // skip BOM
        let u16s: Vec<u16> = output
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let text = String::from_utf16(&u16s).unwrap();
        assert!(text.contains("$KEY1\tZnachenie"));
        assert!(text.contains("$KEY2\tTekst"));
    }
}
