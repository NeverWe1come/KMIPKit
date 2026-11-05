//! Bounded XML event traversal; Profiles v2.1 §5.4.1.6.
use super::characters::{attribute, xml_char, xml_space};
use super::names::tag_value;
use super::values::scalar;
use super::{ALLOCATION, INVALID_DOCUMENT, INVALID_VALUE, LIMIT, NAMESPACE, WireCodecError};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use zeroize::Zeroizing;

struct Frame {
    name: Zeroizing<String>,
    tag: kmipkit_ttlv::Tag,
    value: Option<Value>,
    children: Structure,
    namespaces: Vec<(Zeroizing<String>, Zeroizing<String>)>,
}

fn qname(name: &str) -> Result<(&str, &str), WireCodecError> {
    fn valid(name: &str) -> bool {
        let mut chars = name.chars();
        chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
            && chars.all(|c| c == '_' || c == '-' || c == '.' || c.is_alphanumeric())
    }
    let (prefix, local) = name.split_once(':').unwrap_or(("", name));
    if (!prefix.is_empty() && !valid(prefix)) || !valid(local) || local.contains(':') {
        return Err(INVALID_DOCUMENT);
    }
    Ok((prefix, local))
}

fn start(event: &BytesStart<'_>, frames: &[Frame]) -> Result<Frame, WireCodecError> {
    let name = event.name();
    let name = name.as_ref();
    let (prefix, local) = qname(name)?;
    let mut attributes: [Option<Zeroizing<String>>; 4] = [None, None, None, None];
    let mut namespaces = Vec::new();
    for attr in event.attributes() {
        let attr = attr.map_err(|_| INVALID_DOCUMENT)?;
        let key = attr.key.as_ref();
        if key == "xmlns" || key.starts_with("xmlns:") {
            let namespace_prefix = if key == "xmlns" { "" } else { &key[6..] };
            if !namespace_prefix.is_empty() {
                qname(namespace_prefix)?;
            }
            let uri = attribute(&attr.value)?;
            if namespace_prefix == "xmlns"
                || uri.as_str() == "http://www.w3.org/2000/xmlns/"
                || (namespace_prefix == "xml"
                    && uri.as_str() != "http://www.w3.org/XML/1998/namespace")
                || (namespace_prefix != "xml"
                    && uri.as_str() == "http://www.w3.org/XML/1998/namespace")
                || (!namespace_prefix.is_empty() && uri.is_empty())
            {
                return Err(INVALID_DOCUMENT);
            }
            namespaces.try_reserve(1).map_err(|_| ALLOCATION)?;
            namespaces.push((attribute(namespace_prefix)?, uri));
        } else {
            let index = match key {
                "tag" => 0,
                "name" => 1,
                "type" => 2,
                "value" => 3,
                _ => return Err(INVALID_DOCUMENT),
            };
            if attributes[index].is_some() {
                return Err(INVALID_DOCUMENT);
            }
            attributes[index] = Some(attribute(&attr.value)?);
        }
    }
    let namespace = namespaces
        .iter()
        .rev()
        .chain(
            frames
                .iter()
                .rev()
                .flat_map(|frame| frame.namespaces.iter().rev()),
        )
        .find(|(p, _)| p.as_str() == prefix)
        .map(|(_, uri)| uri.as_str());
    if prefix == "xml"
        || match namespace {
            Some("") | None => !prefix.is_empty(),
            Some(uri) => uri != NAMESPACE,
        }
    {
        return Err(INVALID_DOCUMENT);
    }
    let raw = if local == "TTLV" {
        tag_value(
            attributes[0]
                .as_deref()
                .map(String::as_str)
                .ok_or(INVALID_DOCUMENT)?,
        )?
    } else {
        if attributes[0].is_some() || attributes[1].is_some() {
            return Err(INVALID_DOCUMENT);
        }
        tag_value(local)?
    };
    let tag = RawTag::new(raw)
        .and_then(|t| t.try_checked())
        .map_err(|_| INVALID_VALUE)?;
    let ty = attributes[2].as_deref().map_or("Structure", String::as_str);
    let value = if ty == "Structure" {
        if attributes[3].is_some() {
            return Err(INVALID_DOCUMENT);
        }
        None
    } else {
        Some(scalar(
            raw,
            ty,
            attributes[3]
                .as_deref()
                .map(String::as_str)
                .ok_or(INVALID_DOCUMENT)?,
        )?)
    };
    Ok(Frame {
        name: attribute(name)?,
        tag,
        value,
        children: Structure::new(),
        namespaces,
    })
}

fn finish(frame: Frame) -> Result<Item, WireCodecError> {
    Item::new(
        frame.tag,
        frame
            .value
            .unwrap_or_else(|| Value::structure(frame.children)),
    )
    .map_err(|_| INVALID_VALUE)
}

fn attach(item: Item, frames: &mut [Frame], root: &mut Option<Item>) -> Result<(), WireCodecError> {
    if let Some(parent) = frames.last_mut() {
        parent.children.try_push(item).map_err(|_| INVALID_VALUE)
    } else if root.is_some() {
        Err(INVALID_DOCUMENT)
    } else {
        *root = Some(item);
        Ok(())
    }
}

fn preflight(
    event: &BytesStart<'_>,
    depth: usize,
    limits: &CodecLimits,
) -> Result<(), WireCodecError> {
    if depth < limits.max_structure_depth() {
        return Ok(());
    }
    // Only type metadata is needed to decide whether this element adds a
    // Structure. Never decode its payload or namespace attributes first.
    for attr in event.attributes().with_checks(false) {
        let attr = attr.map_err(|_| INVALID_DOCUMENT)?;
        if attr.key.as_ref() == "type" {
            return if attribute(&attr.value)?.as_str() == "Structure" {
                Err(LIMIT)
            } else {
                Ok(())
            };
        }
    }
    Err(LIMIT)
}

fn check_declaration(text: &str) -> Result<(), WireCodecError> {
    let declaration = BytesStart::from_content(text, 3);
    let mut state = 0;
    for attribute in declaration.attributes() {
        let attribute = attribute.map_err(|_| INVALID_DOCUMENT)?;
        match (state, attribute.key.as_ref(), attribute.value.as_ref()) {
            (0, "version", "1.0") => state = 1,
            (1, "encoding", encoding) if encoding.eq_ignore_ascii_case("UTF-8") => state = 2,
            (1 | 2, "standalone", "yes" | "no") => state = 3,
            _ => return Err(INVALID_DOCUMENT),
        }
    }
    if state == 0 {
        return Err(INVALID_DOCUMENT);
    }
    Ok(())
}

pub(crate) fn decode(bytes: &[u8], limits: &CodecLimits) -> Result<Item, WireCodecError> {
    if bytes.len() > limits.max_message_bytes() {
        return Err(LIMIT);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| INVALID_DOCUMENT)?;
    if !text.chars().all(xml_char) {
        return Err(INVALID_DOCUMENT);
    }
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_comments = true;
    // Compare our own zeroizing names rather than copying to a parser stack.
    reader.config_mut().check_end_names = false;
    let mut frames: Vec<Frame> = Vec::new();
    let mut root = None;
    let mut count = 0;
    let mut declaration = false;
    loop {
        let before = reader.buffer_position();
        let event = reader.read_event().map_err(|_| INVALID_DOCUMENT)?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(event) | Event::Empty(event) => {
                if count >= limits.max_elements() {
                    return Err(LIMIT);
                }
                if frames.last().is_some_and(|f| f.value.is_some())
                    || (frames.is_empty() && root.is_some())
                {
                    return Err(INVALID_DOCUMENT);
                }
                // Every ancestor is a Structure; preflight before attributes
                // or model payloads are allocated for this item.
                if frames.len() > limits.max_structure_depth() {
                    return Err(LIMIT);
                }
                count += 1;
                preflight(&event, frames.len(), limits)?;
                let frame = start(&event, &frames)?;
                if frame.value.is_none() && frames.len() >= limits.max_structure_depth() {
                    return Err(LIMIT);
                }
                if empty {
                    attach(finish(frame)?, &mut frames, &mut root)?;
                } else {
                    frames.try_reserve(1).map_err(|_| ALLOCATION)?;
                    frames.push(frame);
                }
            }
            Event::End(event) => {
                let frame = frames.pop().ok_or(INVALID_DOCUMENT)?;
                if event.name().as_ref() != frame.name.as_str() {
                    return Err(INVALID_DOCUMENT);
                }
                attach(finish(frame)?, &mut frames, &mut root)?;
            }
            Event::Text(event) if event.as_ref().chars().all(xml_space) => {}
            Event::CData(event) if !frames.is_empty() && event.as_ref().chars().all(xml_space) => {}
            Event::Comment(_) => {}
            Event::PI(event) => {
                if event.target().eq_ignore_ascii_case("xml") {
                    return Err(INVALID_DOCUMENT);
                }
            }
            Event::Decl(event) => {
                if declaration || before != 0 || count != 0 || root.is_some() {
                    return Err(INVALID_DOCUMENT);
                }
                check_declaration(event.as_ref())?;
                declaration = true;
            }
            Event::Eof => {
                if !frames.is_empty() {
                    return Err(INVALID_DOCUMENT);
                }
                return root.ok_or(INVALID_DOCUMENT);
            }
            _ => return Err(INVALID_DOCUMENT),
        }
    }
}
