//! Wire format codecs for TTLV, JSON, and XML.

pub mod datetime;
pub mod json;
pub mod names;
pub mod ttlv;
pub mod xml;

use kmipkit_ttlv::Item;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Ttlv,
    Json,
    Xml,
}

impl Format {
    /// Detects format from HTTP content-type header or raw payload bytes.
    pub fn detect(content_type: Option<&str>, bytes: &[u8]) -> Self {
        if let Some(ct) = content_type {
            let lower = ct.to_ascii_lowercase();
            if lower.contains("application/json") {
                return Self::Json;
            }
            if lower.contains("text/xml") || lower.contains("application/xml") {
                return Self::Xml;
            }
            if lower.contains("application/octet-stream") {
                return Self::Ttlv;
            }
        }

        // Sniff leading bytes
        let trimmed = bytes
            .iter()
            .copied()
            .skip_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
            .collect::<Vec<_>>();

        if let Some(&first) = trimmed.first() {
            if first == b'{' {
                return Self::Json;
            }
            if first == b'<' {
                return Self::Xml;
            }
        }

        // Default to TTLV
        Self::Ttlv
    }

    /// Content-Type string for HTTP responses.
    pub const fn content_type(self) -> &'static str {
        match self {
            Self::Ttlv => "application/octet-stream",
            Self::Json => "application/json",
            Self::Xml => "text/xml",
        }
    }

    /// Decodes message bytes according to format.
    pub fn decode(self, bytes: &[u8]) -> Result<Item, String> {
        match self {
            Self::Ttlv => ttlv::decode_ttlv(bytes).map_err(|e| format!("TTLV decode error: {e}")),
            Self::Json => json::decode_json(bytes),
            Self::Xml => xml::decode_xml(bytes),
        }
    }

    /// Encodes item into message bytes according to format.
    pub fn encode(self, item: &Item) -> Result<Vec<u8>, String> {
        match self {
            Self::Ttlv => Ok(ttlv::encode_ttlv(item)),
            Self::Json => json::encode_json(item).map(String::into_bytes),
            Self::Xml => xml::encode_xml(item).map(String::into_bytes),
        }
    }
}
