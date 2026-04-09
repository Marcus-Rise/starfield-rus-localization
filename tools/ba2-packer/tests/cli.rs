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
        .stdout(predicate::str::contains("rename"));
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
