use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
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
        .stdout(predicate::str::contains("repack"));
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
