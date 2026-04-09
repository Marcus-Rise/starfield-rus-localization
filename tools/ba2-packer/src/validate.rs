use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const EXPECTED_STRING_FILES: &[&str] = &[
    "starfield_en.STRINGS",
    "starfield_en.DLSTRINGS",
    "starfield_en.ILSTRINGS",
    "blueprintships-starfield_en.STRINGS",
    "blueprintships-starfield_en.DLSTRINGS",
    "blueprintships-starfield_en.ILSTRINGS",
    "constellation_en.STRINGS",
    "constellation_en.DLSTRINGS",
    "constellation_en.ILSTRINGS",
    "oldmars_en.STRINGS",
    "oldmars_en.DLSTRINGS",
    "oldmars_en.ILSTRINGS",
];

const MAX_MOD_SIZE_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GB

struct ValidationResult {
    check: String,
    passed: bool,
    detail: String,
}

impl ValidationResult {
    fn pass(check: &str) -> Self {
        Self {
            check: check.to_string(),
            passed: true,
            detail: "OK".to_string(),
        }
    }

    fn fail(check: &str, detail: &str) -> Self {
        Self {
            check: check.to_string(),
            passed: false,
            detail: detail.to_string(),
        }
    }
}

/// Check ESM has ESM flag set (bit 0 of record flags at offset 8)
fn check_esm_flag(esm_data: &[u8]) -> ValidationResult {
    let check = "ESM flag set";
    if esm_data.len() < 12 {
        return ValidationResult::fail(check, "File too small for ESM header");
    }
    let flags = u32::from_le_bytes([esm_data[8], esm_data[9], esm_data[10], esm_data[11]]);
    if flags & 0x0001 != 0 {
        ValidationResult::pass(check)
    } else {
        ValidationResult::fail(
            check,
            &format!("ESM flag not set in record flags: {flags:#010x}"),
        )
    }
}

/// Check ESM has Localized Strings flag (0x80)
fn check_localized_flag(esm_data: &[u8]) -> ValidationResult {
    let check = "Localized Strings flag (0x80)";
    if esm_data.len() < 12 {
        return ValidationResult::fail(check, "File too small for ESM header");
    }
    let flags = u32::from_le_bytes([esm_data[8], esm_data[9], esm_data[10], esm_data[11]]);
    if flags & 0x0080 != 0 {
        ValidationResult::pass(check)
    } else {
        ValidationResult::fail(
            check,
            &format!("Localized flag not set in record flags: {flags:#010x}"),
        )
    }
}

/// Check ESM HEDR version = 0.96 (stored as f32 at offset 24)
fn check_hedr_version(esm_data: &[u8]) -> ValidationResult {
    let check = "HEDR version = 0.96";
    // TES4 record: type(4) + size(4) + flags(4) + formid(4) + vc(4) + version(2) + vc2(2) = 24
    // Then HEDR subrecord: type(4) + size(2) + version_f32(4)
    let hedr_offset = 24; // start of subrecords
    if esm_data.len() < hedr_offset + 10 {
        return ValidationResult::fail(check, "File too small for HEDR subrecord");
    }

    // Verify HEDR subrecord type
    let sub_type = &esm_data[hedr_offset..hedr_offset + 4];
    if sub_type != b"HEDR" {
        return ValidationResult::fail(
            check,
            &format!("Expected HEDR subrecord, got {sub_type:?}"),
        );
    }

    let version_offset = hedr_offset + 6; // after type(4) + size(2)
    let version = f32::from_le_bytes([
        esm_data[version_offset],
        esm_data[version_offset + 1],
        esm_data[version_offset + 2],
        esm_data[version_offset + 3],
    ]);

    if (version - 0.96).abs() < 0.001 {
        ValidationResult::pass(check)
    } else {
        ValidationResult::fail(check, &format!("HEDR version is {version}, expected 0.96"))
    }
}

/// Check ESM references Starfield.esm as master
fn check_master_reference(esm_data: &[u8]) -> ValidationResult {
    let check = "Master reference: Starfield.esm";

    // Search for MAST subrecord containing "Starfield.esm"
    let needle = b"Starfield.esm";
    let found = esm_data
        .windows(needle.len())
        .any(|window| window == needle);

    if found {
        ValidationResult::pass(check)
    } else {
        ValidationResult::fail(check, "Starfield.esm not found as master reference")
    }
}

/// Check string file header parses correctly
fn check_string_file(data: &[u8], filename: &str) -> ValidationResult {
    let check_name = format!("String file valid: {filename}");
    if data.len() < 8 {
        return ValidationResult::fail(&check_name, "File too small for header (need 8 bytes)");
    }

    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let _data_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

    // Directory: count * 8 bytes (id: u32 + offset: u32)
    let directory_size = count * 8;
    let expected_min = 8 + directory_size;

    if data.len() < expected_min {
        return ValidationResult::fail(
            &check_name,
            &format!(
                "File too small: has {} bytes, need at least {expected_min} for {count} entries",
                data.len()
            ),
        );
    }

    ValidationResult::pass(&check_name)
}

/// Check `translate_en.txt` is UTF-16LE with BOM
fn check_translate_encoding(data: &[u8]) -> ValidationResult {
    let check = "translate_en.txt: UTF-16LE with BOM";
    if data.len() < 2 {
        return ValidationResult::fail(check, "File too small for BOM");
    }
    if data[0] == 0xFF && data[1] == 0xFE {
        ValidationResult::pass(check)
    } else {
        ValidationResult::fail(
            check,
            &format!(
                "Expected BOM 0xFF 0xFE, got {:#04x} {:#04x}",
                data[0], data[1]
            ),
        )
    }
}

/// Check `translate_en.txt` has `$KEY\tValue` format
fn check_translate_format(data: &[u8]) -> ValidationResult {
    let check = "translate_en.txt: $KEY\\tValue format";
    if data.len() < 2 {
        return ValidationResult::fail(check, "File too small");
    }

    // Skip BOM
    let content = &data[2..];

    // Decode UTF-16LE
    let (text, _, had_errors) = encoding_rs::UTF_16LE.decode(content);
    if had_errors {
        return ValidationResult::fail(check, "Failed to decode as UTF-16LE");
    }

    for (i, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('$') {
            return ValidationResult::fail(
                check,
                &format!("Line {} does not start with '$': {}", i + 1, line),
            );
        }
        if !line.contains('\t') {
            return ValidationResult::fail(
                check,
                &format!("Line {} missing tab separator: {}", i + 1, line),
            );
        }
    }

    ValidationResult::pass(check)
}

/// Check `fontconfig_en.txt` references `fontlib "fonts_en"`
fn check_fontconfig_fontlib(data: &[u8]) -> ValidationResult {
    let check = "fontconfig_en.txt: fontlib \"fonts_en\"";
    let text = String::from_utf8_lossy(data);

    if text.contains("fontlib \"fonts_en\"") || text.contains("fontlib \"fonts_en\"") {
        ValidationResult::pass(check)
    } else {
        ValidationResult::fail(check, "fontlib \"fonts_en\" not found in fontconfig")
    }
}

/// Check `fontconfig_en.txt` `validNameChars` contains Cyrillic range
fn check_fontconfig_cyrillic(data: &[u8]) -> ValidationResult {
    let check = "fontconfig_en.txt: Cyrillic in validNameChars";
    let text = String::from_utf8_lossy(data);

    // Check for at least some Cyrillic characters
    let has_cyrillic =
        text.contains('А') && text.contains('Я') && text.contains('а') && text.contains('я');

    if has_cyrillic {
        ValidationResult::pass(check)
    } else {
        ValidationResult::fail(check, "Cyrillic characters not found in validNameChars")
    }
}

/// Check `fonts_en.swf` is a valid SWF file (magic bytes)
fn check_swf_magic(data: &[u8]) -> ValidationResult {
    let check = "fonts_en.swf: valid SWF magic";
    if data.len() < 3 {
        return ValidationResult::fail(check, "File too small for SWF header");
    }

    let magic = &data[0..3];
    match magic {
        b"FWS" | b"CWS" | b"ZWS" => ValidationResult::pass(check),
        _ => ValidationResult::fail(
            check,
            &format!("Invalid SWF magic: {magic:?} (expected FWS/CWS/ZWS)"),
        ),
    }
}

/// Check BA2 archive has valid Starfield format header
fn check_ba2_header(data: &[u8], filename: &str) -> ValidationResult {
    let check_name = format!("BA2 valid: {filename}");
    if data.len() < 8 {
        return ValidationResult::fail(&check_name, "File too small for BA2 header");
    }

    // BA2 magic: "BTDX"
    let magic = &data[0..4];
    if magic != b"BTDX" {
        return ValidationResult::fail(
            &check_name,
            &format!("Invalid BA2 magic: {magic:?} (expected BTDX)"),
        );
    }

    // Version: u32 at offset 4
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version == 2 || version == 3 {
        ValidationResult::pass(&check_name)
    } else {
        ValidationResult::fail(
            &check_name,
            &format!("BA2 version {version}, expected 2 or 3 for Starfield"),
        )
    }
}

/// Check total mod size < 2 GB
fn check_total_size(dist_dir: &Path) -> ValidationResult {
    let check = "Total mod size < 2 GB";

    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(dist_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }

    if total < MAX_MOD_SIZE_BYTES {
        let mb = total / (1024 * 1024);
        ValidationResult::pass(&format!("{check} ({mb} MB)"))
    } else {
        #[allow(clippy::cast_precision_loss)]
        let gb = total as f64 / (1024.0 * 1024.0 * 1024.0);
        ValidationResult::fail(check, &format!("Total size {gb:.2} GB exceeds 2 GB limit"))
    }
}

fn collect_checks(dist_dir: &Path) -> Result<Vec<ValidationResult>> {
    let mut results: Vec<ValidationResult> = Vec::new();

    // ESM validation
    let esm_path = dist_dir.join("StarfieldRussian.esm");
    if esm_path.exists() {
        let esm_data = fs::read(&esm_path).context("Failed to read ESM file")?;
        results.push(check_esm_flag(&esm_data));
        results.push(check_localized_flag(&esm_data));
        results.push(check_hedr_version(&esm_data));
        results.push(check_master_reference(&esm_data));
    } else {
        results.push(ValidationResult::fail(
            "ESM file exists",
            "StarfieldRussian.esm not found",
        ));
    }

    // String files check (inside BA2 or loose)
    let strings_dir = dist_dir.join("Strings");
    if strings_dir.is_dir() {
        for filename in EXPECTED_STRING_FILES {
            let path = strings_dir.join(filename);
            if path.exists() {
                let data = fs::read(&path)?;
                results.push(check_string_file(&data, filename));
            } else {
                results.push(ValidationResult::fail(
                    &format!("String file present: {filename}"),
                    "File not found",
                ));
            }
        }
    } else {
        results.push(ValidationResult::fail(
            "All 12 string files present",
            "Strings directory not found in dist",
        ));
    }

    // Interface files
    let interface_dir = dist_dir.join("Interface");
    collect_interface_checks(&mut results, dist_dir, &interface_dir)?;

    // BA2 archives
    for (name, path) in [
        (
            "StarfieldRussian - Main.ba2",
            dist_dir.join("StarfieldRussian - Main.ba2"),
        ),
        (
            "StarfieldRussian - Interface.ba2",
            dist_dir.join("StarfieldRussian - Interface.ba2"),
        ),
    ] {
        if path.exists() {
            let data = fs::read(&path)?;
            results.push(check_ba2_header(&data, name));
        }
    }

    results.push(check_total_size(dist_dir));
    Ok(results)
}

fn collect_interface_checks(
    results: &mut Vec<ValidationResult>,
    dist_dir: &Path,
    interface_dir: &Path,
) -> Result<()> {
    // translate_en.txt
    let translate_data = read_with_fallback(interface_dir, dist_dir, "translate_en.txt")?;
    if let Some(data) = translate_data {
        results.push(check_translate_encoding(&data));
        results.push(check_translate_format(&data));
    }

    // fontconfig_en.txt
    let fontconfig_data = read_with_fallback(interface_dir, dist_dir, "fontconfig_en.txt")?;
    if let Some(data) = fontconfig_data {
        results.push(check_fontconfig_fontlib(&data));
        results.push(check_fontconfig_cyrillic(&data));
    }

    // fonts_en.swf
    let swf_data = read_with_fallback(interface_dir, dist_dir, "fonts_en.swf")?;
    if let Some(data) = swf_data {
        results.push(check_swf_magic(&data));
    }

    Ok(())
}

fn read_with_fallback(primary: &Path, fallback: &Path, filename: &str) -> Result<Option<Vec<u8>>> {
    let primary_path = primary.join(filename);
    if primary_path.exists() {
        return Ok(Some(fs::read(&primary_path)?));
    }
    let fallback_path = fallback.join(filename);
    if fallback_path.exists() {
        return Ok(Some(fs::read(&fallback_path)?));
    }
    Ok(None)
}

pub fn run(dist_dir: &Path) -> Result<()> {
    if !dist_dir.is_dir() {
        anyhow::bail!("Dist directory does not exist: {}", dist_dir.display());
    }

    let results = collect_checks(dist_dir)?;

    // Print results
    let mut failed = 0;
    for r in &results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        let icon = if r.passed { "\u{2713}" } else { "\u{2717}" };
        println!("[{icon} {status}] {} — {}", r.check, r.detail);
        if !r.passed {
            failed += 1;
        }
    }

    println!(
        "\n{}/{} checks passed",
        results.len() - failed,
        results.len()
    );

    if failed > 0 {
        anyhow::bail!("{failed} validation check(s) failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_esm_flag_valid() {
        // TES4 type + size + flags with ESM bit set
        let mut data = vec![0u8; 32];
        data[0..4].copy_from_slice(b"TES4");
        data[8] = 0x81; // ESM (0x01) + Localized (0x80)
        let result = check_esm_flag(&data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_esm_flag_missing() {
        let mut data = vec![0u8; 32];
        data[0..4].copy_from_slice(b"TES4");
        data[8] = 0x80; // Only Localized, no ESM
        let result = check_esm_flag(&data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_localized_flag_valid() {
        let mut data = vec![0u8; 32];
        data[8] = 0x81;
        let result = check_localized_flag(&data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_localized_flag_missing() {
        let mut data = vec![0u8; 32];
        data[8] = 0x01; // Only ESM, no Localized
        let result = check_localized_flag(&data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_hedr_version_valid() {
        let mut data = vec![0u8; 40];
        data[0..4].copy_from_slice(b"TES4");
        // HEDR subrecord at offset 24
        data[24..28].copy_from_slice(b"HEDR");
        data[28..30].copy_from_slice(&12u16.to_le_bytes()); // subrecord size
                                                            // 0.96 as f32
        let version_bytes = 0.96_f32.to_le_bytes();
        data[30..34].copy_from_slice(&version_bytes);
        let result = check_hedr_version(&data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_hedr_version_wrong() {
        let mut data = vec![0u8; 40];
        data[0..4].copy_from_slice(b"TES4");
        data[24..28].copy_from_slice(b"HEDR");
        data[28..30].copy_from_slice(&12u16.to_le_bytes());
        let version_bytes = 1.0_f32.to_le_bytes();
        data[30..34].copy_from_slice(&version_bytes);
        let result = check_hedr_version(&data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_master_reference_found() {
        let mut data = b"TES4\x00\x00\x00\x00\x81\x00\x00\x00".to_vec();
        data.extend_from_slice(b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
        data.extend_from_slice(b"MAST\x0E\x00Starfield.esm\x00");
        let result = check_master_reference(&data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_master_reference_missing() {
        let data = vec![0u8; 100];
        let result = check_master_reference(&data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_string_file_valid() {
        // Header: count=1, data_size=5
        // Directory: id=1, offset=0
        // Data: "test\0"
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes()); // count
        data.extend_from_slice(&5u32.to_le_bytes()); // data_size
        data.extend_from_slice(&1u32.to_le_bytes()); // id
        data.extend_from_slice(&0u32.to_le_bytes()); // offset
        data.extend_from_slice(b"test\0"); // string data
        let result = check_string_file(&data, "test.STRINGS");
        assert!(result.passed);
    }

    #[test]
    fn test_check_string_file_too_small() {
        let data = vec![0u8; 4]; // only 4 bytes, need 8
        let result = check_string_file(&data, "test.STRINGS");
        assert!(!result.passed);
    }

    #[test]
    fn test_check_translate_encoding_valid() {
        let data = vec![0xFF, 0xFE, 0x24, 0x00]; // BOM + '$'
        let result = check_translate_encoding(&data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_translate_encoding_wrong_bom() {
        let data = vec![0xFE, 0xFF, 0x24, 0x00]; // Wrong BOM (big-endian)
        let result = check_translate_encoding(&data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_translate_format_valid() {
        // UTF-16LE BOM + "$KEY\tValue\n"
        let mut data = vec![0xFF, 0xFE];
        for c in "$KEY\tValue\n".encode_utf16() {
            data.extend_from_slice(&c.to_le_bytes());
        }
        let result = check_translate_format(&data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_translate_format_missing_dollar() {
        let mut data = vec![0xFF, 0xFE];
        for c in "KEY\tValue\n".encode_utf16() {
            data.extend_from_slice(&c.to_le_bytes());
        }
        let result = check_translate_format(&data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_fontconfig_fontlib_valid() {
        let data = b"fontlib \"fonts_en\"\nmap \"$MAIN\" = \"Font\" Normal";
        let result = check_fontconfig_fontlib(data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_fontconfig_fontlib_missing() {
        let data = b"fontlib \"fonts_ru\"";
        let result = check_fontconfig_fontlib(data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_fontconfig_cyrillic_valid() {
        let data = "validNameChars \"АБВЯабвя\"".as_bytes();
        let result = check_fontconfig_cyrillic(data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_fontconfig_cyrillic_missing() {
        let data = b"validNameChars \"ABCDabcd\"";
        let result = check_fontconfig_cyrillic(data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_swf_magic_fws() {
        let data = b"FWS\x09\x00\x00\x00\x00";
        let result = check_swf_magic(data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_swf_magic_cws() {
        let data = b"CWS\x09\x00\x00\x00\x00";
        let result = check_swf_magic(data);
        assert!(result.passed);
    }

    #[test]
    fn test_check_swf_magic_invalid() {
        let data = b"PDF-1.5";
        let result = check_swf_magic(data);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_ba2_header_valid_v2() {
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(b"BTDX");
        data[4..8].copy_from_slice(&2u32.to_le_bytes());
        let result = check_ba2_header(&data, "test.ba2");
        assert!(result.passed);
    }

    #[test]
    fn test_check_ba2_header_valid_v3() {
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(b"BTDX");
        data[4..8].copy_from_slice(&3u32.to_le_bytes());
        let result = check_ba2_header(&data, "test.ba2");
        assert!(result.passed);
    }

    #[test]
    fn test_check_ba2_header_wrong_version() {
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(b"BTDX");
        data[4..8].copy_from_slice(&1u32.to_le_bytes()); // FO4 version, not SF
        let result = check_ba2_header(&data, "test.ba2");
        assert!(!result.passed);
    }

    #[test]
    fn test_check_ba2_header_wrong_magic() {
        let data = b"BSA\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = check_ba2_header(data, "test.ba2");
        assert!(!result.passed);
    }
}
