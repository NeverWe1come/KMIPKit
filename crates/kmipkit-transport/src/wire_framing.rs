//! Allocation-free framing scanners for one raw-TLS JSON or XML document.
//!
//! These scanners locate a lexical document boundary only. The KMIP codecs
//! remain responsible for validating JSON/XML grammar and KMIP semantics.

use std::io;

use crate::config::WireEncoding;

#[derive(Debug)]
pub(super) struct TextDocumentFramer {
    state: TextDocumentState,
}

#[derive(Debug)]
enum TextDocumentState {
    Json(JsonDocumentFramer),
    Xml(XmlDocumentFramer),
}

impl TextDocumentFramer {
    pub(super) fn new(encoding: WireEncoding) -> io::Result<Self> {
        match encoding {
            WireEncoding::Json => Ok(Self {
                state: TextDocumentState::Json(JsonDocumentFramer::default()),
            }),
            WireEncoding::Xml => Ok(Self {
                state: TextDocumentState::Xml(XmlDocumentFramer::default()),
            }),
            WireEncoding::Ttlv => Err(invalid_document()),
        }
    }

    pub(super) fn feed(&mut self, byte: u8) -> io::Result<()> {
        match &mut self.state {
            TextDocumentState::Json(framer) => framer.feed(byte),
            TextDocumentState::Xml(framer) => framer.feed(byte),
        }
    }

    pub(super) fn is_complete(&self) -> bool {
        match &self.state {
            TextDocumentState::Json(framer) => framer.is_complete(),
            TextDocumentState::Xml(framer) => framer.is_complete(),
        }
    }
}

#[derive(Debug, Default)]
struct JsonDocumentFramer {
    depth: usize,
    state: JsonState,
}

#[derive(Debug, Default, Eq, PartialEq)]
enum JsonState {
    #[default]
    Leading,
    Structure,
    String,
    Escape,
    Complete,
}

impl JsonDocumentFramer {
    fn feed(&mut self, byte: u8) -> io::Result<()> {
        match self.state {
            JsonState::Leading | JsonState::Complete if json_space(byte) => Ok(()),
            JsonState::Leading if byte == b'{' => {
                self.depth = 1;
                self.state = JsonState::Structure;
                Ok(())
            }
            JsonState::Leading if matches!(byte, b'<' | 0x42 | 0xef) => Err(encoding_mismatch()),
            JsonState::Leading | JsonState::Complete => Err(invalid_document()),
            JsonState::Structure => match byte {
                b'"' => {
                    self.state = JsonState::String;
                    Ok(())
                }
                b'{' | b'[' => {
                    self.depth = self.depth.checked_add(1).ok_or_else(invalid_document)?;
                    Ok(())
                }
                b'}' | b']' => {
                    self.depth = self.depth.checked_sub(1).ok_or_else(invalid_document)?;
                    if self.depth == 0 {
                        self.state = JsonState::Complete;
                    }
                    Ok(())
                }
                _ => Ok(()),
            },
            JsonState::String => match byte {
                b'\\' => {
                    self.state = JsonState::Escape;
                    Ok(())
                }
                b'"' => {
                    self.state = JsonState::Structure;
                    Ok(())
                }
                0x00..=0x1f => Err(invalid_document()),
                _ => Ok(()),
            },
            JsonState::Escape => {
                self.state = JsonState::String;
                Ok(())
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.state == JsonState::Complete
    }
}

fn json_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

#[derive(Debug, Default)]
struct XmlDocumentFramer {
    depth: usize,
    complete: bool,
    state: XmlState,
}

#[derive(Debug, Default)]
enum XmlState {
    #[default]
    Start,
    Bom(u8),
    Text,
    LessThan,
    Bang,
    BangHyphen,
    Comment(u8),
    CdataStart(usize),
    Cdata(u8),
    Processing(bool),
    StartTag {
        quote: Option<u8>,
        slash: bool,
    },
    EndTag,
}

impl XmlDocumentFramer {
    #[allow(clippy::too_many_lines)] // Keep the allocation-free XML byte transitions together.
    fn feed(&mut self, byte: u8) -> io::Result<()> {
        let state = std::mem::take(&mut self.state);
        self.state = match state {
            XmlState::Start => match byte {
                b'<' => XmlState::LessThan,
                0xef => XmlState::Bom(1),
                byte if xml_space(byte) => XmlState::Text,
                b'{' | 0x42 => return Err(encoding_mismatch()),
                _ => return Err(invalid_document()),
            },
            XmlState::Bom(1) if byte == 0xbb => XmlState::Bom(2),
            XmlState::Bom(2) if byte == 0xbf => XmlState::Text,
            XmlState::Bom(_) => return Err(invalid_document()),
            XmlState::Text => {
                if byte == b'<' {
                    XmlState::LessThan
                } else if self.depth > 0 || xml_space(byte) {
                    XmlState::Text
                } else if matches!(byte, b'{' | 0x42) {
                    return Err(encoding_mismatch());
                } else {
                    return Err(invalid_document());
                }
            }
            XmlState::LessThan => match byte {
                b'/' if self.depth > 0 && !self.complete => XmlState::EndTag,
                b'/' => return Err(invalid_document()),
                b'?' => XmlState::Processing(false),
                b'!' => XmlState::Bang,
                _ if self.complete => return Err(invalid_document()),
                _ => XmlState::StartTag {
                    quote: None,
                    slash: false,
                },
            },
            XmlState::Bang => match byte {
                b'-' => XmlState::BangHyphen,
                b'[' if !self.complete && self.depth > 0 => XmlState::CdataStart(0),
                _ => return Err(invalid_document()),
            },
            XmlState::BangHyphen => {
                if byte == b'-' {
                    XmlState::Comment(0)
                } else {
                    return Err(invalid_document());
                }
            }
            XmlState::Comment(dashes) => {
                if byte == b'>' && dashes >= 2 {
                    XmlState::Text
                } else if byte == b'-' {
                    XmlState::Comment(dashes.saturating_add(1).min(2))
                } else {
                    XmlState::Comment(0)
                }
            }
            XmlState::CdataStart(matched) => {
                const PREFIX: &[u8] = b"CDATA[";
                if PREFIX.get(matched) == Some(&byte) {
                    if matched + 1 == PREFIX.len() {
                        XmlState::Cdata(0)
                    } else {
                        XmlState::CdataStart(matched + 1)
                    }
                } else {
                    return Err(invalid_document());
                }
            }
            XmlState::Cdata(brackets) => {
                if byte == b'>' && brackets >= 2 {
                    XmlState::Text
                } else if byte == b']' {
                    XmlState::Cdata(brackets.saturating_add(1).min(2))
                } else {
                    XmlState::Cdata(0)
                }
            }
            XmlState::Processing(question) => {
                if byte == b'>' && question {
                    XmlState::Text
                } else {
                    XmlState::Processing(byte == b'?')
                }
            }
            XmlState::StartTag {
                mut quote,
                mut slash,
            } => {
                match (quote, byte) {
                    (Some(open), close) if open == close => quote = None,
                    (None, b'\'' | b'"') => quote = Some(byte),
                    (None, b'>') => {
                        if slash {
                            if self.depth == 0 {
                                self.complete = true;
                            }
                        } else {
                            self.depth = self.depth.checked_add(1).ok_or_else(invalid_document)?;
                        }
                        self.state = XmlState::Text;
                        return Ok(());
                    }
                    (None, b'/') => slash = true,
                    (None, byte) if !xml_space(byte) => slash = false,
                    _ => {}
                }
                XmlState::StartTag { quote, slash }
            }
            XmlState::EndTag => {
                if byte == b'>' {
                    self.depth = self.depth.checked_sub(1).ok_or_else(invalid_document)?;
                    if self.depth == 0 {
                        self.complete = true;
                    }
                    XmlState::Text
                } else {
                    XmlState::EndTag
                }
            }
        };
        Ok(())
    }

    fn is_complete(&self) -> bool {
        self.complete && matches!(self.state, XmlState::Text)
    }
}

fn xml_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn invalid_document() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid text document framing")
}

fn encoding_mismatch() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "response encoding does not match selected encoding",
    )
}

#[cfg(test)]
#[path = "../tests/unit/wire_framing_tests.rs"]
mod tests;
