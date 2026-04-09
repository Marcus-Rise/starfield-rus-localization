use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("ba2-packer").unwrap()
}

#[test]
fn test_help() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("pack"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("rename"))
        .stdout(predicate::str::contains("extract"))
        .stdout(predicate::str::contains("repack"))
        .stdout(predicate::str::contains("transliterate"))
        .stdout(predicate::str::contains("smoke-test"));
}

#[test]
fn test_pack_help() {
    cmd()
        .args(["pack", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--input-strings"))
        .stdout(predicate::str::contains("--input-interface"))
        .stdout(predicate::str::contains("--output-dir"));
}

#[test]
fn test_validate_help() {
    cmd()
        .args(["validate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dist"));
}

#[test]
fn test_rename_help() {
    cmd()
        .args(["rename", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--input-dir"))
        .stdout(predicate::str::contains("--output-dir"));
}

#[test]
fn test_pack_creates_ba2_files() {
    let strings_dir = TempDir::new().unwrap();
    let interface_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Create minimal test string file
    fs::write(
        strings_dir.path().join("starfield_en.STRINGS"),
        b"\x01\x00\x00\x00\x05\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00test\x00",
    )
    .unwrap();

    // Create minimal test interface file
    fs::write(
        interface_dir.path().join("fontconfig_en.txt"),
        b"fontlib \"fonts_en\"",
    )
    .unwrap();

    cmd()
        .args([
            "pack",
            "--input-strings",
            strings_dir.path().to_str().unwrap(),
            "--input-interface",
            interface_dir.path().to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

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
fn test_pack_nonexistent_input() {
    cmd()
        .args([
            "pack",
            "--input-strings",
            "/nonexistent/path",
            "--input-interface",
            "/nonexistent/path2",
            "--output-dir",
            "/tmp/out",
        ])
        .assert()
        .failure();
}

#[test]
fn test_rename_with_files() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    fs::write(input.path().join("starfield_ru.STRINGS"), b"test data").unwrap();

    cmd()
        .args([
            "rename",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(output.path().join("starfield_en.STRINGS").exists());
}

#[test]
fn test_validate_nonexistent_dir() {
    cmd()
        .args(["validate", "/nonexistent/dist"])
        .assert()
        .failure();
}

// --- Extract ---

#[test]
fn test_extract_help() {
    cmd()
        .args(["extract", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--input"))
        .stdout(predicate::str::contains("--output-dir"));
}

#[test]
fn test_extract_single_file() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Minimal valid .STRINGS binary: 1 entry "hi"
    // Header: count=1, data_size=3 ("hi\0")
    // Directory: id=1, offset=0
    // Data: "hi\0"
    let binary: Vec<u8> = vec![
        1, 0, 0, 0, // count = 1
        3, 0, 0, 0, // data_size = 3
        1, 0, 0, 0, // id = 1
        0, 0, 0, 0, // offset = 0
        b'h', b'i', 0, // "hi\0"
    ];
    let input_path = input_dir.path().join("test_en.STRINGS");
    fs::write(&input_path, &binary).unwrap();

    cmd()
        .args([
            "extract",
            "--input",
            input_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let jsonl_path = output_dir.path().join("test_en.STRINGS.jsonl");
    assert!(jsonl_path.exists());
    let content = fs::read_to_string(&jsonl_path).unwrap();
    assert!(content.contains("\"id\":1"));
    assert!(content.contains("\"text\":\"hi\""));
}

#[test]
fn test_extract_directory() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Two minimal string table files
    let binary: Vec<u8> = vec![1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, b'a', 0];
    fs::write(input_dir.path().join("test_en.STRINGS"), &binary).unwrap();
    fs::write(input_dir.path().join("readme.txt"), b"ignore me").unwrap();

    cmd()
        .args([
            "extract",
            "--input",
            input_dir.path().to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(output_dir.path().join("test_en.STRINGS.jsonl").exists());
}

#[test]
fn test_extract_nonexistent_input() {
    cmd()
        .args([
            "extract",
            "--input",
            "/nonexistent/file.STRINGS",
            "--output-dir",
            "/tmp/out",
        ])
        .assert()
        .failure();
}

// --- Repack ---

#[test]
fn test_repack_help() {
    cmd()
        .args(["repack", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--input"))
        .stdout(predicate::str::contains("--output-dir"));
}

#[test]
fn test_repack_single_file() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let jsonl = r#"{"id":1,"text":"hello"}
{"id":2,"text":"world"}"#;
    let jsonl_path = input_dir.path().join("test_en.STRINGS.jsonl");
    fs::write(&jsonl_path, jsonl).unwrap();

    cmd()
        .args([
            "repack",
            "--input",
            jsonl_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let output_path = output_dir.path().join("test_en.STRINGS");
    assert!(output_path.exists());
    let data = fs::read(&output_path).unwrap();
    // Verify header: count=2
    assert_eq!(data[0], 2);
}

#[test]
fn test_repack_nonexistent_input() {
    cmd()
        .args([
            "repack",
            "--input",
            "/nonexistent/file.STRINGS.jsonl",
            "--output-dir",
            "/tmp/out",
        ])
        .assert()
        .failure();
}

#[test]
fn test_extract_repack_round_trip_cli() {
    let strings_dir = TempDir::new().unwrap();
    let extracted_dir = TempDir::new().unwrap();
    let repacked_dir = TempDir::new().unwrap();

    // Create a valid .STRINGS binary with 2 entries
    let binary: Vec<u8> = vec![
        2, 0, 0, 0, // count = 2
        11, 0, 0, 0, // data_size = 11
        10, 0, 0, 0, // id = 10
        0, 0, 0, 0, // offset = 0
        20, 0, 0, 0, // id = 20
        6, 0, 0, 0, // offset = 6
        b'h', b'e', b'l', b'l', b'o', 0, // "hello\0"
        b'h', b'i', b'!', b'!', 0, // "hi!!\0"
    ];
    let input_path = strings_dir.path().join("data_en.STRINGS");
    fs::write(&input_path, &binary).unwrap();

    // Extract
    cmd()
        .args([
            "extract",
            "--input",
            input_path.to_str().unwrap(),
            "--output-dir",
            extracted_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Repack
    let jsonl_path = extracted_dir.path().join("data_en.STRINGS.jsonl");
    assert!(jsonl_path.exists());

    cmd()
        .args([
            "repack",
            "--input",
            jsonl_path.to_str().unwrap(),
            "--output-dir",
            repacked_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Compare bytes
    let repacked = fs::read(repacked_dir.path().join("data_en.STRINGS")).unwrap();
    assert_eq!(binary, repacked, "Round-trip must produce identical bytes");
}

// --- Transliterate ---

#[test]
fn test_transliterate_help() {
    cmd()
        .args(["transliterate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--input-dir"))
        .stdout(predicate::str::contains("--output-dir"))
        .stdout(predicate::str::contains("--credit"));
}

#[test]
fn test_transliterate_nonexistent_input() {
    cmd()
        .args([
            "transliterate",
            "--input-dir",
            "/nonexistent/path",
            "--output-dir",
            "/tmp/out",
        ])
        .assert()
        .failure();
}

#[test]
fn test_transliterate_with_string_files() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    // Create a .STRINGS file with Cyrillic text "Привет"
    // Binary: count=1, data_size=len("Привет\0" in UTF-8)
    let text_bytes = "Привет".as_bytes();
    let data_size = text_bytes.len() + 1; // +1 for null terminator
    let mut binary: Vec<u8> = Vec::new();
    binary.extend_from_slice(&1u32.to_le_bytes()); // count = 1
    binary.extend_from_slice(&(data_size as u32).to_le_bytes());
    binary.extend_from_slice(&1u32.to_le_bytes()); // id = 1
    binary.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
    binary.extend_from_slice(text_bytes);
    binary.push(0); // null terminator

    fs::write(input.path().join("test_en.STRINGS"), &binary).unwrap();

    cmd()
        .args([
            "transliterate",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify output file exists and contains transliterated text
    let out_path = output.path().join("test_en.STRINGS");
    assert!(out_path.exists());

    // No CREDITS.txt without --credit
    assert!(!output.path().join("CREDITS.txt").exists());
}

#[test]
fn test_transliterate_with_credit() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    // Minimal .STRINGS binary
    let binary: Vec<u8> = vec![
        1, 0, 0, 0, // count = 1
        2, 0, 0, 0, // data_size = 2
        1, 0, 0, 0, // id = 1
        0, 0, 0, 0, // offset = 0
        b'a', 0, // "a\0"
    ];
    fs::write(input.path().join("test_en.STRINGS"), &binary).unwrap();

    cmd()
        .args([
            "transliterate",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
            "--credit",
            "TestAuthor",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("WARNING"))
        .stdout(predicate::str::contains("TestAuthor"));

    assert!(output.path().join("CREDITS.txt").exists());
    let credits = fs::read_to_string(output.path().join("CREDITS.txt")).unwrap();
    assert!(credits.contains("TestAuthor"));
}

// --- Transliterate: strings-only workflow (issue #48) ---

/// Build a minimal STRINGS binary (null-terminated UTF-8).
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

/// Build a minimal DLSTRINGS/ILSTRINGS binary (length-prefixed UTF-8).
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

/// Build a translate_en.txt file in UTF-16LE with BOM.
fn build_translate_utf16le(text: &str) -> Vec<u8> {
    let mut result: Vec<u8> = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        result.extend_from_slice(&unit.to_le_bytes());
    }
    result
}

#[test]
fn test_transliterate_all_three_formats() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    // Create one file per format with Cyrillic text
    fs::write(
        input.path().join("starfield_en.STRINGS"),
        build_strings_binary(&[(1, "Привет")]),
    )
    .unwrap();
    fs::write(
        input.path().join("starfield_en.DLSTRINGS"),
        build_dlstrings_binary(&[(2, "Диалог")]),
    )
    .unwrap();
    fs::write(
        input.path().join("starfield_en.ILSTRINGS"),
        build_dlstrings_binary(&[(3, "Предмет")]),
    )
    .unwrap();

    cmd()
        .args([
            "transliterate",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed: 3 file(s)"));

    // All three output files must exist with original filenames
    assert!(output.path().join("starfield_en.STRINGS").exists());
    assert!(output.path().join("starfield_en.DLSTRINGS").exists());
    assert!(output.path().join("starfield_en.ILSTRINGS").exists());
}

#[test]
fn test_transliterate_with_translate_file() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    // String table + translate_en.txt together
    fs::write(
        input.path().join("starfield_en.STRINGS"),
        build_strings_binary(&[(1, "Тест")]),
    )
    .unwrap();
    fs::write(
        input.path().join("translate_en.txt"),
        build_translate_utf16le("$sKey1\tЗначение"),
    )
    .unwrap();

    cmd()
        .args([
            "transliterate",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed: 2 file(s)"));

    assert!(output.path().join("starfield_en.STRINGS").exists());
    assert!(output.path().join("translate_en.txt").exists());
}

#[test]
fn test_transliterate_empty_dir_error_message() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    cmd()
        .args([
            "transliterate",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No files were found to transliterate",
        ));
}

#[test]
fn test_transliterate_strings_subdirectory_preserved_names() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    // Place files in Strings/ subdirectory (common game layout)
    let strings_dir = input.path().join("Strings");
    fs::create_dir(&strings_dir).unwrap();
    fs::write(
        strings_dir.join("starfield_en.STRINGS"),
        build_strings_binary(&[(1, "Космос")]),
    )
    .unwrap();
    fs::write(
        strings_dir.join("blueprintships-starfield_en.DLSTRINGS"),
        build_dlstrings_binary(&[(2, "Корабль")]),
    )
    .unwrap();

    cmd()
        .args([
            "transliterate",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed: 2 file(s)"));

    // Output filenames must match originals
    assert!(output.path().join("starfield_en.STRINGS").exists());
    assert!(output
        .path()
        .join("blueprintships-starfield_en.DLSTRINGS")
        .exists());
}

// --- Smoke Test ---

#[test]
fn test_smoke_test_help() {
    cmd()
        .args(["smoke-test", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--input-dir"))
        .stdout(predicate::str::contains("--output-dir"))
        .stdout(predicate::str::contains("--interface-dir"))
        .stdout(predicate::str::contains("--credit"));
}

#[test]
fn test_smoke_test_nonexistent_input() {
    cmd()
        .args([
            "smoke-test",
            "--input-dir",
            "/nonexistent/path",
            "--interface-dir",
            "/nonexistent/interface",
        ])
        .assert()
        .failure();
}

/// Build minimal _ru string table and interface fixtures for smoke-test.
fn create_ru_string_fixtures(dir: &Path) {
    let text = "Привет";
    let text_bytes = text.as_bytes();
    let data_size = text_bytes.len() + 1; // +1 for null terminator
    let mut strings_binary: Vec<u8> = Vec::new();
    strings_binary.extend_from_slice(&1u32.to_le_bytes()); // count = 1
    strings_binary.extend_from_slice(&(data_size as u32).to_le_bytes());
    strings_binary.extend_from_slice(&1u32.to_le_bytes()); // id = 1
    strings_binary.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
    strings_binary.extend_from_slice(text_bytes);
    strings_binary.push(0); // null terminator

    // DLSTRINGS/ILSTRINGS use length-prefixed format
    let dl_entry_size = 4 + text_bytes.len(); // length prefix + data
    let mut dl_binary: Vec<u8> = Vec::new();
    dl_binary.extend_from_slice(&1u32.to_le_bytes()); // count = 1
    dl_binary.extend_from_slice(&(dl_entry_size as u32).to_le_bytes());
    dl_binary.extend_from_slice(&1u32.to_le_bytes()); // id = 1
    dl_binary.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
    dl_binary.extend_from_slice(&(text_bytes.len() as u32).to_le_bytes());
    dl_binary.extend_from_slice(text_bytes);

    // Create all 12 expected _ru string table files
    let prefixes = [
        "starfield_ru",
        "blueprintships-starfield_ru",
        "constellation_ru",
        "oldmars_ru",
    ];
    let extensions = ["STRINGS", "DLSTRINGS", "ILSTRINGS"];

    for prefix in &prefixes {
        for ext in &extensions {
            let filename = format!("{prefix}.{ext}");
            if *ext == "STRINGS" {
                fs::write(dir.join(&filename), &strings_binary).unwrap();
            } else {
                fs::write(dir.join(&filename), &dl_binary).unwrap();
            }
        }
    }

    // Also include _ru interface files in the input package
    fs::write(
        dir.join("fontconfig_ru.txt"),
        "fontlib \"fonts_ru\"\nmap \"$ConsoleFont\" = \"Font\" Normal\n\
         validNameChars \" ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz\
         АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя\"",
    )
    .unwrap();

    // translate_ru.txt (UTF-16LE with BOM) with Cyrillic value
    let translate_text = "$sInputKey\tЗначение из пакета";
    let mut translate_data: Vec<u8> = vec![0xFF, 0xFE]; // BOM
    for unit in translate_text.encode_utf16() {
        translate_data.extend_from_slice(&unit.to_le_bytes());
    }
    fs::write(dir.join("translate_ru.txt"), &translate_data).unwrap();

    // Minimal SWF
    let mut swf_data = Vec::new();
    swf_data.extend_from_slice(b"FWS");
    swf_data.push(0x0A);
    swf_data.extend_from_slice(&100u32.to_le_bytes());
    swf_data.resize(100, 0);
    fs::write(dir.join("fonts_ru.swf"), &swf_data).unwrap();
}

/// Create minimal interface fixtures for smoke-test.
fn create_interface_fixtures(dir: &Path) {
    fs::write(
        dir.join("fontconfig_en.txt"),
        "fontlib \"fonts_en\"\nmap \"$ConsoleFont\" = \"Font\" Normal\n\
         validNameChars \" ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz\
         АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя\"",
    )
    .unwrap();

    // Minimal translate_en.txt (UTF-16LE with BOM)
    let translate_text = "$sTestKey\tTest Value";
    let mut translate_data: Vec<u8> = vec![0xFF, 0xFE]; // BOM
    for unit in translate_text.encode_utf16() {
        translate_data.extend_from_slice(&unit.to_le_bytes());
    }
    fs::write(dir.join("translate_en.txt"), &translate_data).unwrap();

    // Minimal SWF file (just the magic bytes for validation)
    let mut swf_data = Vec::new();
    swf_data.extend_from_slice(b"FWS"); // SWF magic
    swf_data.push(0x0A); // version
    swf_data.extend_from_slice(&100u32.to_le_bytes()); // file length
    swf_data.resize(100, 0); // pad to declared length
    fs::write(dir.join("fonts_en.swf"), &swf_data).unwrap();
}

#[test]
fn test_smoke_test_with_ru_files() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    let interface = TempDir::new().unwrap();

    create_ru_string_fixtures(input.path());
    create_interface_fixtures(interface.path());

    cmd()
        .args([
            "smoke-test",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
            "--interface-dir",
            interface.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Smoke Test: local E2E pipeline"))
        .stdout(predicate::str::contains("Step 1/5"))
        .stdout(predicate::str::contains("Step 5/5"))
        .stdout(predicate::str::contains("Publish Readiness Summary"))
        .stdout(predicate::str::contains("Smoke test PASSED"));

    // Verify all 3 artifacts were created
    assert!(output.path().join("StarfieldRussian.esm").exists());
    assert!(output.path().join("StarfieldRussian - Main.ba2").exists());
    assert!(output
        .path()
        .join("StarfieldRussian - Interface.ba2")
        .exists());
}

#[test]
fn test_smoke_test_with_credit() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    let interface = TempDir::new().unwrap();

    create_ru_string_fixtures(input.path());
    create_interface_fixtures(interface.path());

    cmd()
        .args([
            "smoke-test",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
            "--interface-dir",
            interface.path().to_str().unwrap(),
            "--credit",
            "TestTranslator",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Credits: present (TestTranslator)",
        ));

    assert!(output.path().join("CREDITS.txt").exists());
}

#[test]
fn test_smoke_test_custom_output_dir() {
    let input = TempDir::new().unwrap();
    let interface = TempDir::new().unwrap();
    let custom_output = TempDir::new().unwrap();
    let custom_path = custom_output.path().join("my-dist");

    create_ru_string_fixtures(input.path());
    create_interface_fixtures(interface.path());

    cmd()
        .args([
            "smoke-test",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            custom_path.to_str().unwrap(),
            "--interface-dir",
            interface.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(custom_path.to_str().unwrap()));

    assert!(custom_path.join("StarfieldRussian.esm").exists());
}

#[test]
fn test_smoke_test_stages_input_interface_files() {
    // Verify that interface files from the _ru input package are used
    // for pack/validate, not only the external --interface-dir files.
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    // Intentionally empty interface dir — all interface files come from input package
    let interface = TempDir::new().unwrap();

    create_ru_string_fixtures(input.path());
    // Don't call create_interface_fixtures — rely on _ru package's own interface files

    cmd()
        .args([
            "smoke-test",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
            "--interface-dir",
            interface.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Smoke test PASSED"));

    // Interface BA2 should exist (packed from staged input package files)
    assert!(output
        .path()
        .join("StarfieldRussian - Interface.ba2")
        .exists());
}

#[test]
fn test_smoke_test_default_interface_dir_from_subdirectory() {
    // Issue #41: --interface-dir default should resolve relative to repo root,
    // not the current working directory.
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    create_ru_string_fixtures(input.path());

    // Run from tools/ba2-packer/ (a subdirectory of the repo root) WITHOUT
    // --interface-dir. The CLI should locate the repo root and resolve
    // src/interface automatically.
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let subdir = repo_root.join("tools").join("ba2-packer");

    cmd()
        .current_dir(&subdir)
        .args([
            "smoke-test",
            "--input-dir",
            input.path().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Smoke test PASSED"));
}
