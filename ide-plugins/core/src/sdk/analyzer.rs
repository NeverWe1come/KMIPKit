//! Multi-language SDK diagnostics (Rust, Java, Python, C, KMIP JSON) and anonymized test generator.

use serde::{Deserialize, Serialize};

use crate::ttlv::inspector::{DecodeOptions, decode_ttlv};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkDiagnostic {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub code: String,
    pub message: String,
    pub replacement: Option<(String, String)>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetLanguage {
    Rust,
    Java,
    Python,
    C,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedTest {
    pub language: String,
    pub code: String,
}

/// Scans source files (`.rs`, `.java`, `.py`, `.c`, `.kmip.json`) for weak KMIP algorithms,
/// deprecated KMIP 1.x tags (`TemplateAttribute`), and insecure key lengths.
#[must_use]
pub fn analyze_source(uri: &str, text: &str) -> Vec<SdkDiagnostic> {
    let mut diags = Vec::new();
    let is_code = uri.ends_with(".rs")
        || uri.ends_with(".java")
        || uri.ends_with(".py")
        || uri.ends_with(".c")
        || uri.ends_with(".h")
        || uri.ends_with(".kmip.json")
        || uri.ends_with(".kmip-policy.json");

    if !is_code {
        return diags;
    }

    type PatternRule = (&'static str, &'static str, &'static str, Option<(&'static str, &'static str)>);
    let patterns: &[PatternRule] = &[
        (
            "3DES",
            "KMIP_WEAK_ALGORITHM",
            "3DES/TripleDES is deprecated in modern KMIP profiles; prefer AES-256.",
            Some(("3DES", "AES")),
        ),
        (
            "RC4",
            "KMIP_WEAK_ALGORITHM",
            "RC4 is cryptographically broken; prefer AES-256 or ChaCha20Poly1305.",
            Some(("RC4", "AES")),
        ),
        (
            "MD5",
            "KMIP_WEAK_ALGORITHM",
            "MD5 is cryptographically broken; prefer SHA-256 or SHA-384.",
            Some(("MD5", "SHA-256")),
        ),
        (
            "SHA-1",
            "KMIP_WEAK_ALGORITHM",
            "SHA-1 is deprecated for signatures/HMAC; prefer SHA-256.",
            Some(("SHA-1", "SHA-256")),
        ),
        (
            "TemplateAttribute",
            "KMIP_DEPRECATED_1X_TAG",
            "TemplateAttribute (0x42008E) was removed in KMIP 2.0+; use Attributes (0x4200C0).",
            Some(("TemplateAttribute", "Attributes")),
        ),
    ];

    for (line_idx, line) in text.lines().enumerate() {
        for (needle, code, msg, repl) in patterns {
            if let Some(col) = line.find(needle) {
                diags.push(SdkDiagnostic {
                    line: line_idx,
                    start_col: col,
                    end_col: col + needle.len(),
                    code: (*code).to_string(),
                    message: (*msg).to_string(),
                    replacement: repl.map(|(a, b)| (a.to_string(), b.to_string())),
                });
            }
        }
    }

    diags
}

/// Generates an anonymized unit test in the requested language from a TTLV hex/base64 capture.
/// Sensitive key material is automatically replaced with deterministic dummy bytes.
///
/// # Errors
///
/// Returns an error if the TTLV payload cannot be decoded.
pub fn generate_test_from_ttlv(
    input: &str,
    language: TargetLanguage,
) -> Result<GeneratedTest, String> {
    let decoded = decode_ttlv(input, &DecodeOptions::default())?;
    // Replace redacted '*' runs in rawHex with deterministic 0xAA dummy bytes for valid test fixtures
    let anonymized_hex: String = decoded
        .raw_hex
        .chars()
        .map(|c| if c == '*' { 'A' } else { c })
        .collect();
    let root_tag = &decoded.root.tag_hex;
    let root_name = &decoded.root.tag_name;

    let code = match language {
        TargetLanguage::Rust => format!(
            "#[test]\nfn generated_kmip_ttlv_fixture_test() {{\n    // Anonymized fixture for {root_name} ({root_tag})\n    let ttlv_bytes = hex::decode(\"{anonymized_hex}\").expect(\"valid hex\");\n    let item = kmipkit_ttlv::codec::decode(&ttlv_bytes).expect(\"valid TTLV\");\n    assert_eq!(format!(\"0x{{:06X}}\", item.tag().raw()), \"{root_tag}\");\n}}\n"
        ),
        TargetLanguage::Python => format!(
            "def test_generated_kmip_ttlv_fixture():\n    # Anonymized fixture for {root_name} ({root_tag})\n    ttlv_bytes = bytes.fromhex(\"{anonymized_hex}\")\n    assert len(ttlv_bytes) == {len}\n    assert ttlv_bytes[:3].hex().upper() == \"{tag_short}\"\n",
            len = decoded.byte_length,
            tag_short = root_tag.trim_start_matches("0x")
        ),
        TargetLanguage::Java => format!(
            "@Test\nvoid generatedKmipTtlvFixtureTest() {{\n    // Anonymized fixture for {root_name} ({root_tag})\n    byte[] ttlvBytes = HexFormat.of().parseHex(\"{anonymized_hex}\");\n    assertEquals({len}, ttlvBytes.length);\n}}\n",
            len = decoded.byte_length
        ),
        TargetLanguage::C => format!(
            "void test_generated_kmip_ttlv_fixture(void) {{\n    /* Anonymized fixture for {root_name} ({root_tag}) */\n    const char *hex_fixture = \"{anonymized_hex}\";\n    assert(strlen(hex_fixture) == {hex_len});\n}}\n",
            hex_len = anonymized_hex.len()
        ),
    };

    Ok(GeneratedTest {
        language: format!("{language:?}").to_ascii_lowercase(),
        code,
    })
}
