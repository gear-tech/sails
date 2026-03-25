//! Embed and extract Sails IDL from WASM custom sections.
//!
//! This crate provides functions to inject IDL text into a WASM binary
//! as a custom section named `sails:idl`, and to extract it back.
//!
//! # Custom Section Format
//!
//! ```text
//! Offset   Field     Size       Description
//! 0        version   1 byte     Envelope format version (0x01)
//! 1        flags     1 byte     Bit 0: compression (0=raw UTF-8, 1=deflate)
//! 2..N     data      variable   IDL text (raw or deflate-compressed)
//! ```

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use std::path::Path;

const SECTION_NAME: &str = "sails:idl";
const ENVELOPE_VERSION: u8 = 0x01;
const FLAG_COMPRESSED: u8 = 0x01;
const MAX_DECOMPRESSED_SIZE: usize = 1024 * 1024; // 1 MB

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("WASM parsing error: {0}")]
    WasmParse(#[from] wasmparser::BinaryReaderError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("deflate decompression failed: {0}")]
    Deflate(std::io::Error),
    #[error("decompressed IDL exceeds maximum size of {MAX_DECOMPRESSED_SIZE} bytes")]
    DecompressionBomb,
    #[error("IDL data is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Embed IDL text into a WASM binary as a `sails:idl` custom section.
///
/// Returns the modified WASM bytes. If the WASM already contains a `sails:idl`
/// section, it is replaced (idempotent). If `idl` is empty, returns the
/// original WASM unchanged (no section added).
///
/// IDL is compressed with raw deflate by default.
pub fn embed_idl(wasm_bytes: &[u8], idl: &str) -> Result<Vec<u8>> {
    if idl.is_empty() {
        return Ok(wasm_bytes.to_vec());
    }

    // Compress IDL with deflate
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(idl.as_bytes())?;
    let compressed = encoder.finish()?;

    // Build envelope: version + flags + data
    let mut payload = Vec::with_capacity(2 + compressed.len());
    payload.push(ENVELOPE_VERSION);
    payload.push(FLAG_COMPRESSED);
    payload.extend_from_slice(&compressed);

    // Parse existing WASM and rebuild without any existing sails:idl section
    let parser = wasmparser::Parser::new(0);
    let mut module = wasm_encoder::Module::new();

    for section in parser.parse_all(wasm_bytes) {
        let section = section?;
        match section {
            wasmparser::Payload::CustomSection(custom) if custom.name() == SECTION_NAME => {
                // Skip existing section (will be replaced)
                continue;
            }
            wasmparser::Payload::End(_) => break,
            wasmparser::Payload::Version { .. } => {
                // Skip the version header — wasm_encoder::Module handles this
                continue;
            }
            other => {
                if let Some((id, range)) = other.as_section() {
                    module.section(&wasm_encoder::RawSection {
                        id,
                        data: &wasm_bytes[range],
                    });
                }
            }
        }
    }

    // Append the new sails:idl custom section
    module.section(&wasm_encoder::CustomSection {
        name: SECTION_NAME.into(),
        data: payload.into(),
    });

    Ok(module.finish())
}

/// Extract IDL text from a WASM binary's `sails:idl` custom section.
///
/// Returns `Ok(None)` if no `sails:idl` section is found, or if the section
/// has an unknown envelope version (forward compatibility).
///
/// Returns `Err` if the section exists but is corrupted (deflate failure,
/// invalid UTF-8, or decompressed data exceeds 1 MB).
pub fn extract_idl(wasm_bytes: &[u8]) -> Result<Option<String>> {
    let parser = wasmparser::Parser::new(0);

    for section in parser.parse_all(wasm_bytes) {
        let section = section?;
        if let wasmparser::Payload::CustomSection(custom) = section {
            if custom.name() == SECTION_NAME {
                let data = custom.data();
                if data.len() < 2 {
                    return Ok(None);
                }

                let version = data[0];
                if version != ENVELOPE_VERSION {
                    // Unknown version — skip gracefully
                    return Ok(None);
                }

                let flags = data[1];
                let content = &data[2..];

                let idl_bytes = if flags & FLAG_COMPRESSED != 0 {
                    decompress_with_limit(content)?
                } else {
                    if content.len() > MAX_DECOMPRESSED_SIZE {
                        return Err(Error::DecompressionBomb);
                    }
                    content.to_vec()
                };

                let idl = String::from_utf8(idl_bytes)?;
                return Ok(Some(idl));
            }
        }
    }

    Ok(None)
}

/// Embed IDL into a WASM file in-place.
///
/// Reads the WASM file, embeds the IDL, and writes back. If `idl` is empty,
/// the file is not modified.
pub fn embed_idl_to_file(wasm_path: &Path, idl: &str) -> Result<()> {
    if idl.is_empty() {
        return Ok(());
    }
    let wasm_bytes = std::fs::read(wasm_path)?;
    let modified = embed_idl(&wasm_bytes, idl)?;
    std::fs::write(wasm_path, modified)?;
    Ok(())
}

/// Extract IDL from a WASM file.
///
/// Returns `Ok(None)` if no `sails:idl` section is found.
pub fn extract_idl_from_file(wasm_path: &Path) -> Result<Option<String>> {
    let wasm_bytes = std::fs::read(wasm_path)?;
    extract_idl(&wasm_bytes)
}

fn decompress_with_limit(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut buf = Vec::new();
    // Read in chunks to detect decompression bombs early
    let mut chunk = [0u8; 8192];
    loop {
        let n = decoder.read(&mut chunk).map_err(Error::Deflate)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.len() > MAX_DECOMPRESSED_SIZE {
            return Err(Error::DecompressionBomb);
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid WASM module (empty module).
    fn minimal_wasm() -> Vec<u8> {
        wasm_encoder::Module::new().finish()
    }

    #[test]
    fn round_trip_embed_extract() {
        let wasm = minimal_wasm();
        let idl = "service Ping { functions { Ping() -> String; } }";

        let embedded = embed_idl(&wasm, idl).unwrap();
        let extracted = extract_idl(&embedded).unwrap();

        assert_eq!(extracted, Some(idl.to_string()));
    }

    #[test]
    fn empty_idl_skips_embedding() {
        let wasm = minimal_wasm();
        let result = embed_idl(&wasm, "").unwrap();
        assert_eq!(result, wasm);

        let extracted = extract_idl(&result).unwrap();
        assert_eq!(extracted, None);
    }

    #[test]
    fn no_section_returns_none() {
        let wasm = minimal_wasm();
        let extracted = extract_idl(&wasm).unwrap();
        assert_eq!(extracted, None);
    }

    #[test]
    fn unknown_version_returns_none() {
        let wasm = minimal_wasm();
        // Manually inject a section with version 0x02
        let mut module = wasm_encoder::Module::new();
        let payload = vec![0x02, 0x00, b'h', b'i'];
        module.section(&wasm_encoder::CustomSection {
            name: SECTION_NAME.into(),
            data: payload.into(),
        });
        let wasm_with_unknown = module.finish();

        let extracted = extract_idl(&wasm_with_unknown).unwrap();
        assert_eq!(extracted, None);
    }

    #[test]
    fn corrupted_deflate_returns_err() {
        // Inject a section with compression flag but invalid deflate data
        let mut module = wasm_encoder::Module::new();
        let payload = vec![ENVELOPE_VERSION, FLAG_COMPRESSED, 0xFF, 0xFE, 0xFD];
        module.section(&wasm_encoder::CustomSection {
            name: SECTION_NAME.into(),
            data: payload.into(),
        });
        let wasm = module.finish();

        let result = extract_idl(&wasm);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_utf8_returns_err() {
        // Compress invalid UTF-8 bytes
        let invalid_bytes: &[u8] = &[0xFF, 0xFE, 0x80, 0x81];
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(invalid_bytes).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut module = wasm_encoder::Module::new();
        let mut payload = vec![ENVELOPE_VERSION, FLAG_COMPRESSED];
        payload.extend_from_slice(&compressed);
        module.section(&wasm_encoder::CustomSection {
            name: SECTION_NAME.into(),
            data: payload.into(),
        });
        let wasm = module.finish();

        let result = extract_idl(&wasm);
        assert!(result.is_err());
    }

    #[test]
    fn decompression_bomb_returns_err() {
        // Create a deflate stream that decompresses to >1MB
        // Compressing 2MB of zeros produces a tiny compressed output
        let big_data = vec![0u8; 2 * 1024 * 1024];
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&big_data).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut module = wasm_encoder::Module::new();
        let mut payload = vec![ENVELOPE_VERSION, FLAG_COMPRESSED];
        payload.extend_from_slice(&compressed);
        module.section(&wasm_encoder::CustomSection {
            name: SECTION_NAME.into(),
            data: payload.into(),
        });
        let wasm = module.finish();

        let result = extract_idl(&wasm);
        assert!(matches!(result, Err(Error::DecompressionBomb)));
    }

    #[test]
    fn idempotent_re_embedding() {
        let wasm = minimal_wasm();
        let idl1 = "service V1 {}";
        let idl2 = "service V2 { functions { Ping() -> String; } }";

        let embedded1 = embed_idl(&wasm, idl1).unwrap();
        let embedded2 = embed_idl(&embedded1, idl2).unwrap();

        let extracted = extract_idl(&embedded2).unwrap();
        assert_eq!(extracted, Some(idl2.to_string()));
    }

    #[test]
    fn file_operations_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let wasm_path = dir.path().join("test.wasm");
        std::fs::write(&wasm_path, minimal_wasm()).unwrap();

        let idl = "service FileTest { functions { Get() -> u32; } }";
        embed_idl_to_file(&wasm_path, idl).unwrap();

        let extracted = extract_idl_from_file(&wasm_path).unwrap();
        assert_eq!(extracted, Some(idl.to_string()));
    }

    #[test]
    fn file_not_found_returns_err() {
        let result = extract_idl_from_file(Path::new("/nonexistent/path.wasm"));
        assert!(result.is_err());
    }

    #[test]
    fn embed_file_not_found_returns_err() {
        let result = embed_idl_to_file(Path::new("/nonexistent/path.wasm"), "test");
        assert!(result.is_err());
    }

    #[test]
    fn large_idl_round_trip() {
        let wasm = minimal_wasm();
        // Generate a ~10KB IDL (realistic upper bound)
        let mut idl = String::from("service Large {\n  functions {\n");
        for i in 0..200 {
            idl.push_str(&format!("    Method{i}(arg: String) -> Result<String, String>;\n"));
        }
        idl.push_str("  }\n}");

        let embedded = embed_idl(&wasm, &idl).unwrap();
        let extracted = extract_idl(&embedded).unwrap();
        assert_eq!(extracted, Some(idl));
    }

    #[test]
    fn section_too_short_returns_none() {
        // Section with only 1 byte (less than envelope minimum)
        let mut module = wasm_encoder::Module::new();
        module.section(&wasm_encoder::CustomSection {
            name: SECTION_NAME.into(),
            data: vec![0x01].into(),
        });
        let wasm = module.finish();

        let extracted = extract_idl(&wasm).unwrap();
        assert_eq!(extracted, None);
    }
}
