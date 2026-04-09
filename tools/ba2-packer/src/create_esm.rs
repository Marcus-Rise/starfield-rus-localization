use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const MOD_NAME: &str = "StarfieldRussian";
const MASTER: &[u8] = b"Starfield.esm\0";

/// Build a minimal Starfield ESM plugin binary.
///
/// Layout (TES4 record):
///   Record header: `type(4)` + `data_size(4)` + `flags(4)` + `formid(4)` + `vc(4)` + `version(2)` + `vc2(2)`
///   HEDR subrecord: `type(4)` + `size(2)` + `version(4)` + `numRecords(4)` + `nextObjectId(4)`
///   MAST subrecord: `type(4)` + `size(2)` + `master_name`(null-terminated)
///   DATA subrecord: `type(4)` + `size(2)` + 8 zero bytes (master data)
fn build_esm() -> Vec<u8> {
    let hedr_data_size: u16 = 12; // f32 + u32 + u32
    let hedr_sub_size = 4 + 2 + hedr_data_size as usize; // 18

    let mast_data_size = u16::try_from(MASTER.len()).expect("MASTER length fits in u16");
    let mast_sub_size = 4 + 2 + mast_data_size as usize;

    let data_data_size: u16 = 8; // u64 zero
    let data_sub_size = 4 + 2 + data_data_size as usize; // 14

    let record_data_size = hedr_sub_size + mast_sub_size + data_sub_size;

    let mut buf = Vec::with_capacity(24 + record_data_size);

    // Record header
    buf.extend_from_slice(b"TES4");
    buf.extend_from_slice(
        &u32::try_from(record_data_size)
            .expect("record data size fits in u32")
            .to_le_bytes(),
    );
    // Flags: ESM (0x01) + Localized (0x80) = 0x81
    buf.extend_from_slice(&0x0000_0081u32.to_le_bytes());
    // FormID
    buf.extend_from_slice(&0u32.to_le_bytes());
    // Version control info
    buf.extend_from_slice(&0u32.to_le_bytes());
    // Internal version: 0x022F (559) for Starfield
    buf.extend_from_slice(&0x022Fu16.to_le_bytes());
    // VC2
    buf.extend_from_slice(&0u16.to_le_bytes());

    // HEDR subrecord
    buf.extend_from_slice(b"HEDR");
    buf.extend_from_slice(&hedr_data_size.to_le_bytes());
    buf.extend_from_slice(&0.96_f32.to_le_bytes()); // version
    buf.extend_from_slice(&0u32.to_le_bytes()); // numRecords
    buf.extend_from_slice(&0x800u32.to_le_bytes()); // nextObjectId

    // MAST subrecord
    buf.extend_from_slice(b"MAST");
    buf.extend_from_slice(&mast_data_size.to_le_bytes());
    buf.extend_from_slice(MASTER);

    // DATA subrecord (master data — 8 zero bytes)
    buf.extend_from_slice(b"DATA");
    buf.extend_from_slice(&data_data_size.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());

    buf
}

pub fn run(output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let esm = build_esm();
    let filename = format!("{MOD_NAME}.esm");
    let output_path = if output.is_dir() {
        output.join(&filename)
    } else {
        output.to_path_buf()
    };

    fs::write(&output_path, &esm)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    println!("Created: {} ({} bytes)", output_path.display(), esm.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_esm_magic() {
        let esm = build_esm();
        assert_eq!(&esm[0..4], b"TES4");
    }

    #[test]
    fn test_build_esm_flags() {
        let esm = build_esm();
        let flags = u32::from_le_bytes([esm[8], esm[9], esm[10], esm[11]]);
        assert_eq!(flags & 0x01, 0x01, "ESM flag must be set");
        assert_eq!(flags & 0x80, 0x80, "Localized flag must be set");
    }

    #[test]
    fn test_build_esm_hedr_version() {
        let esm = build_esm();
        // HEDR at offset 24: type(4) + size(2) + version(4)
        assert_eq!(&esm[24..28], b"HEDR");
        let version = f32::from_le_bytes([esm[30], esm[31], esm[32], esm[33]]);
        assert!((version - 0.96).abs() < 0.001);
    }

    #[test]
    fn test_build_esm_master_reference() {
        let esm = build_esm();
        let has_master = esm
            .windows(b"Starfield.esm".len())
            .any(|w| w == b"Starfield.esm");
        assert!(has_master, "ESM must reference Starfield.esm");
    }

    #[test]
    fn test_build_esm_internal_version() {
        let esm = build_esm();
        let version = u16::from_le_bytes([esm[20], esm[21]]);
        assert_eq!(
            version, 0x022F,
            "Internal version should be 559 (Starfield)"
        );
    }

    #[test]
    fn test_run_creates_file() {
        let dir = TempDir::new().unwrap();
        let output = dir.path().join("StarfieldRussian.esm");
        run(&output).unwrap();
        assert!(output.exists());
        let data = fs::read(&output).unwrap();
        assert_eq!(&data[0..4], b"TES4");
    }

    #[test]
    fn test_run_to_directory() {
        let dir = TempDir::new().unwrap();
        run(dir.path()).unwrap();
        assert!(dir.path().join("StarfieldRussian.esm").exists());
    }

    #[test]
    fn test_esm_passes_validate_checks() {
        // Cross-module: verify the generated ESM passes all 4 validate ESM checks
        let esm = build_esm();

        let flags = u32::from_le_bytes([esm[8], esm[9], esm[10], esm[11]]);
        assert_eq!(flags & 0x01, 0x01, "ESM flag");
        assert_eq!(flags & 0x80, 0x80, "Localized flag");

        assert_eq!(&esm[24..28], b"HEDR");
        let version = f32::from_le_bytes([esm[30], esm[31], esm[32], esm[33]]);
        assert!((version - 0.96).abs() < 0.001, "HEDR version = 0.96");

        assert!(
            esm.windows(b"Starfield.esm".len())
                .any(|w| w == b"Starfield.esm"),
            "Master reference"
        );
    }
}
