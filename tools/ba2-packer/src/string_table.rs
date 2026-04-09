use anyhow::{bail, Context, Result};

/// Type of string table file, determines encoding of string data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringTableType {
    /// `.STRINGS` — null-terminated UTF-8
    Strings,
    /// `.DLSTRINGS` — u32 length-prefixed UTF-8
    DlStrings,
    /// `.ILSTRINGS` — u32 length-prefixed UTF-8
    IlStrings,
}

impl StringTableType {
    pub fn from_extension(ext: &str) -> Result<Self> {
        match ext.to_ascii_uppercase().as_str() {
            "STRINGS" => Ok(Self::Strings),
            "DLSTRINGS" => Ok(Self::DlStrings),
            "ILSTRINGS" => Ok(Self::IlStrings),
            _ => bail!("Unknown string table extension: {ext}"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StringEntry {
    pub id: u32,
    pub text: String,
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let bytes: [u8; 4] = data
        .get(offset..offset + 4)
        .context("Unexpected end of data")?
        .try_into()
        .unwrap();
    Ok(u32::from_le_bytes(bytes))
}

/// Parse a binary string table into a list of `(id, text)` entries.
pub fn parse_string_table(data: &[u8], table_type: StringTableType) -> Result<Vec<StringEntry>> {
    if data.len() < 8 {
        bail!("File too small for header: {} bytes (need 8)", data.len());
    }

    let count = read_u32(data, 0)? as usize;
    let _data_size = read_u32(data, 4)? as usize;

    let dir_end = 8 + count * 8;
    if data.len() < dir_end {
        bail!(
            "File too small for directory: {} bytes (need {dir_end} for {count} entries)",
            data.len()
        );
    }

    let data_base = dir_end;
    let mut entries = Vec::with_capacity(count);

    for i in 0..count {
        let dir_offset = 8 + i * 8;
        let string_id = read_u32(data, dir_offset)?;
        let offset = read_u32(data, dir_offset + 4)? as usize;
        let abs_offset = data_base + offset;

        if abs_offset > data.len() {
            bail!(
                "Offset out of bounds: id={string_id}, offset={offset}, file size={}",
                data.len()
            );
        }

        let text = match table_type {
            StringTableType::Strings => {
                // Null-terminated UTF-8
                let start = abs_offset;
                let end = data[start..]
                    .iter()
                    .position(|&b| b == 0)
                    .map_or(data.len(), |pos| start + pos);
                String::from_utf8(data[start..end].to_vec()).with_context(|| {
                    format!("Invalid UTF-8 in STRINGS entry id={string_id} at offset {offset}")
                })?
            }
            StringTableType::DlStrings | StringTableType::IlStrings => {
                // u32 length-prefixed UTF-8
                let len = read_u32(data, abs_offset)? as usize;
                let start = abs_offset + 4;
                let end = start + len;
                if end > data.len() {
                    bail!("String data overflows file: id={string_id}, offset={offset}, len={len}");
                }
                String::from_utf8(data[start..end].to_vec()).with_context(|| {
                    format!(
                        "Invalid UTF-8 in {table_type:?} entry id={string_id} at offset {offset}"
                    )
                })?
            }
        };

        entries.push(StringEntry {
            id: string_id,
            text,
        });
    }

    Ok(entries)
}

/// Write a list of string entries back into binary string table format.
pub fn write_string_table(entries: &[StringEntry], table_type: StringTableType) -> Vec<u8> {
    // First pass: serialize all string data and record offsets
    let mut data_buf = Vec::new();
    let mut offsets = Vec::with_capacity(entries.len());

    for entry in entries {
        offsets.push(data_buf.len());
        let bytes = entry.text.as_bytes();
        match table_type {
            StringTableType::Strings => {
                data_buf.extend_from_slice(bytes);
                data_buf.push(0); // null terminator
            }
            StringTableType::DlStrings | StringTableType::IlStrings => {
                let len = u32::try_from(bytes.len()).expect("string length fits in u32");
                data_buf.extend_from_slice(&len.to_le_bytes());
                data_buf.extend_from_slice(bytes);
            }
        }
    }

    // Build output
    let count = u32::try_from(entries.len()).expect("entry count fits in u32");
    let data_size = u32::try_from(data_buf.len()).expect("data size fits in u32");

    let mut buf = Vec::with_capacity(8 + entries.len() * 8 + data_buf.len());

    // Header
    buf.extend_from_slice(&count.to_le_bytes());
    buf.extend_from_slice(&data_size.to_le_bytes());

    // Directory
    for (entry, &offset) in entries.iter().zip(&offsets) {
        buf.extend_from_slice(&entry.id.to_le_bytes());
        let off = u32::try_from(offset).expect("offset fits in u32");
        buf.extend_from_slice(&off.to_le_bytes());
    }

    // Data
    buf.extend_from_slice(&data_buf);

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- from_extension ---

    #[test]
    fn test_from_extension_strings() {
        assert_eq!(
            StringTableType::from_extension("STRINGS").unwrap(),
            StringTableType::Strings
        );
    }

    #[test]
    fn test_from_extension_case_insensitive() {
        assert_eq!(
            StringTableType::from_extension("strings").unwrap(),
            StringTableType::Strings
        );
        assert_eq!(
            StringTableType::from_extension("DlStrings").unwrap(),
            StringTableType::DlStrings
        );
    }

    #[test]
    fn test_from_extension_unknown() {
        assert!(StringTableType::from_extension("TXT").is_err());
    }

    // --- Helper to build synthetic binary ---

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

    // --- parse_string_table ---

    #[test]
    fn test_parse_strings_single() {
        let data = build_strings_binary(&[(1, "hello")]);
        let entries = parse_string_table(&data, StringTableType::Strings).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 1);
        assert_eq!(entries[0].text, "hello");
    }

    #[test]
    fn test_parse_strings_multiple() {
        let data = build_strings_binary(&[(10, "aaa"), (20, "bbb"), (30, "ccc")]);
        let entries = parse_string_table(&data, StringTableType::Strings).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].id, 10);
        assert_eq!(entries[0].text, "aaa");
        assert_eq!(entries[2].id, 30);
        assert_eq!(entries[2].text, "ccc");
    }

    #[test]
    fn test_parse_strings_empty() {
        let data = build_strings_binary(&[]);
        let entries = parse_string_table(&data, StringTableType::Strings).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_strings_cyrillic() {
        let data = build_strings_binary(&[(1, "Привет мир")]);
        let entries = parse_string_table(&data, StringTableType::Strings).unwrap();
        assert_eq!(entries[0].text, "Привет мир");
    }

    #[test]
    fn test_parse_strings_with_newlines() {
        let data = build_strings_binary(&[(1, "line1\nline2\ttab")]);
        let entries = parse_string_table(&data, StringTableType::Strings).unwrap();
        assert_eq!(entries[0].text, "line1\nline2\ttab");
    }

    #[test]
    fn test_parse_dlstrings_single() {
        let data = build_dlstrings_binary(&[(42, "test value")]);
        let entries = parse_string_table(&data, StringTableType::DlStrings).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 42);
        assert_eq!(entries[0].text, "test value");
    }

    #[test]
    fn test_parse_ilstrings_single() {
        let data = build_dlstrings_binary(&[(99, "dialogue line")]);
        let entries = parse_string_table(&data, StringTableType::IlStrings).unwrap();
        assert_eq!(entries[0].text, "dialogue line");
    }

    #[test]
    fn test_parse_truncated_header() {
        let data = vec![0u8; 4]; // only 4 bytes
        assert!(parse_string_table(&data, StringTableType::Strings).is_err());
    }

    #[test]
    fn test_parse_truncated_directory() {
        // header says count=2 but no directory data
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        assert!(parse_string_table(&data, StringTableType::Strings).is_err());
    }

    #[test]
    fn test_parse_offset_out_of_bounds() {
        // header says count=1, directory points offset beyond file
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes()); // count = 1
        data.extend_from_slice(&0u32.to_le_bytes()); // data_size = 0
        data.extend_from_slice(&1u32.to_le_bytes()); // id = 1
        data.extend_from_slice(&255u32.to_le_bytes()); // offset = 255 (way beyond)
        assert!(parse_string_table(&data, StringTableType::Strings).is_err());
    }

    // --- write_string_table ---

    #[test]
    fn test_write_strings_single() {
        let entries = vec![StringEntry {
            id: 1,
            text: "hello".to_string(),
        }];
        let data = write_string_table(&entries, StringTableType::Strings);
        // header: count=1, data_size=6 ("hello\0")
        assert_eq!(read_u32(&data, 0).unwrap(), 1);
        assert_eq!(read_u32(&data, 4).unwrap(), 6);
        // directory: id=1, offset=0
        assert_eq!(read_u32(&data, 8).unwrap(), 1);
        assert_eq!(read_u32(&data, 12).unwrap(), 0);
        // data: "hello\0"
        assert_eq!(&data[16..22], b"hello\0");
    }

    #[test]
    fn test_write_dlstrings_single() {
        let entries = vec![StringEntry {
            id: 42,
            text: "test".to_string(),
        }];
        let data = write_string_table(&entries, StringTableType::DlStrings);
        // header: count=1, data_size=8 (4 len + 4 bytes)
        assert_eq!(read_u32(&data, 0).unwrap(), 1);
        assert_eq!(read_u32(&data, 4).unwrap(), 8);
        // data: len=4 then "test"
        assert_eq!(read_u32(&data, 16).unwrap(), 4);
        assert_eq!(&data[20..24], b"test");
    }

    #[test]
    fn test_write_empty() {
        let entries: Vec<StringEntry> = vec![];
        let data = write_string_table(&entries, StringTableType::Strings);
        assert_eq!(data.len(), 8);
        assert_eq!(read_u32(&data, 0).unwrap(), 0);
        assert_eq!(read_u32(&data, 4).unwrap(), 0);
    }

    // --- Round-trip ---

    #[test]
    fn test_round_trip_strings() {
        let original = build_strings_binary(&[(1, "hello"), (2, "world"), (3, "")]);
        let entries = parse_string_table(&original, StringTableType::Strings).unwrap();
        let rebuilt = write_string_table(&entries, StringTableType::Strings);
        assert_eq!(original, rebuilt);
    }

    #[test]
    fn test_round_trip_dlstrings() {
        let original = build_dlstrings_binary(&[(10, "foo"), (20, "bar baz")]);
        let entries = parse_string_table(&original, StringTableType::DlStrings).unwrap();
        let rebuilt = write_string_table(&entries, StringTableType::DlStrings);
        assert_eq!(original, rebuilt);
    }

    #[test]
    fn test_round_trip_ilstrings() {
        let original = build_dlstrings_binary(&[(100, "Привет"), (200, "мир")]);
        let entries = parse_string_table(&original, StringTableType::IlStrings).unwrap();
        let rebuilt = write_string_table(&entries, StringTableType::IlStrings);
        assert_eq!(original, rebuilt);
    }

    #[test]
    fn test_round_trip_empty() {
        let original = build_strings_binary(&[]);
        let entries = parse_string_table(&original, StringTableType::Strings).unwrap();
        let rebuilt = write_string_table(&entries, StringTableType::Strings);
        assert_eq!(original, rebuilt);
    }
}
