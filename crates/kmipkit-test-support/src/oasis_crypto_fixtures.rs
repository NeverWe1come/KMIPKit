//! Test-only adapter for the pinned OASIS Encrypt and Decrypt XML fixtures.
//!
//! The adapter retains each selected message as an ordered generic TTLV tree.
//! It substitutes only the documented fixture symbols after filtering to
//! Encrypt and Decrypt items. It does not validate complete Test Case flows.

use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::fmt;

use kmipkit_ttlv::{Item, RawTag, Structure, Value};
use roxmltree::{Document, Node, NodeType};
use serde_json::Value as JsonValue;

const NORMATIVE_CATALOG: &str = include_str!("../../../specification/catalog/kmip-2.1.json");
const MAX_XML_BYTES: usize = 16 * 1024 * 1024;
const MAX_TTLV_DEPTH: usize = 64;
const MAX_TTLV_ITEMS: usize = 100_000;
const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const TEST_NOW: i64 = 1_700_000_000;
const TEST_UNIQUE_IDENTIFIER_0: &str = "kmipkit-test-unique-id-0";
const TEST_CORRELATION_VALUE: &[u8] = b"kmipkit-test-correlation-value-0";

/// A selected client-side cryptographic operation in an OASIS fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OasisCryptoOperation {
    /// KMIP Encrypt (`0x0000001F`).
    Encrypt,
    /// KMIP Decrypt (`0x00000020`).
    Decrypt,
}

/// Parsed fixture messages selected for the `KMIPKit` Encrypt/Decrypt tests.
#[derive(Debug)]
pub struct OasisCryptoFixture {
    operation_pairs: Vec<OasisCryptoOperationPair>,
}

impl OasisCryptoFixture {
    /// Parses a KMIP XML fixture and retains paired Encrypt/Decrypt messages.
    ///
    /// Source sequence is read from the request's `ClientCorrelationValue`.
    /// The XML test-case stream carries no response sequence field, so a
    /// response is associated with the request at the same source position.
    /// Symbols are checked only after an operation has been selected.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error when the XML, message pairing, selected
    /// operation structure, symbol substitution, limits, or TTLV model is
    /// invalid. Error formatting does not include XML text or attribute values.
    pub fn from_xml(case_id: &str, xml: &str) -> Result<Self, OasisCryptoFixtureError> {
        if xml.len() > MAX_XML_BYTES || contains_doctype_declaration(xml) {
            return Err(OasisCryptoFixtureError::XmlLimitOrDtd);
        }

        let document = Document::parse(xml).map_err(|_| OasisCryptoFixtureError::MalformedXml)?;
        let root = document.root_element();
        if root.tag_name().name() != "KMIP" || case_id.is_empty() {
            return Err(OasisCryptoFixtureError::InvalidFixtureShape);
        }

        let mut requests = Vec::new();
        let mut responses = Vec::new();
        for node in root.children().filter(Node::is_element) {
            match node.tag_name().name() {
                "RequestMessage" => requests.push(request_metadata(node)?),
                "ResponseMessage" => responses.push(response_metadata(node)?),
                _ => return Err(OasisCryptoFixtureError::InvalidFixtureShape),
            }
        }
        if requests.is_empty() || requests.len() != responses.len() {
            return Err(OasisCryptoFixtureError::UnpairedMessages);
        }

        let catalog = CatalogMaps::load()?;
        let mut seen_sequences = HashSet::new();
        let mut operation_pairs = Vec::new();
        for (request, response) in requests.into_iter().zip(responses) {
            let Some(operation) = request.operation else {
                continue;
            };
            if response.operation != Some(operation) {
                return Err(OasisCryptoFixtureError::MismatchedOperations);
            }

            let step_identity = request
                .step_identity
                .ok_or(OasisCryptoFixtureError::InvalidFixtureShape)?;
            let source_sequence = request
                .source_sequence
                .ok_or(OasisCryptoFixtureError::InvalidFixtureShape)?;
            if !seen_sequences.insert(source_sequence) {
                return Err(OasisCryptoFixtureError::DuplicateSequence);
            }

            let request_message = parse_message(request.node, &catalog)?;
            let response_message = parse_message(response.node, &catalog)?;
            operation_pairs.push(OasisCryptoOperationPair {
                case_id: case_id.to_owned(),
                source_sequence,
                step_identity,
                operation,
                request_message,
                response_message,
            });
        }

        Ok(Self { operation_pairs })
    }

    /// Returns the selected operation pairs in fixture source order.
    #[must_use]
    pub fn operation_pairs(&self) -> &[OasisCryptoOperationPair] {
        &self.operation_pairs
    }

    /// Transfers the fixture's selected operation pairs to the caller.
    #[must_use]
    pub fn into_operation_pairs(self) -> Vec<OasisCryptoOperationPair> {
        self.operation_pairs
    }
}

/// One request/response pair selected from the ordered fixture stream.
#[derive(Debug)]
pub struct OasisCryptoOperationPair {
    case_id: String,
    source_sequence: usize,
    step_identity: String,
    operation: OasisCryptoOperation,
    request_message: Structure,
    response_message: Structure,
}

impl OasisCryptoOperationPair {
    /// Returns the supplied OASIS Test Case identifier.
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns the numeric step from the request's `ClientCorrelationValue`.
    #[must_use]
    pub const fn source_sequence(&self) -> usize {
        self.source_sequence
    }

    /// Returns the original case and step identity from the request header.
    #[must_use]
    pub fn step_identity(&self) -> &str {
        &self.step_identity
    }

    /// Returns the selected operation.
    #[must_use]
    pub const fn operation(&self) -> OasisCryptoOperation {
        self.operation
    }

    /// Returns the complete generic request message tree in source order.
    #[must_use]
    pub const fn request_message(&self) -> &Structure {
        &self.request_message
    }

    /// Returns the complete generic response message tree in source order.
    #[must_use]
    pub const fn response_message(&self) -> &Structure {
        &self.response_message
    }

    /// Transfers the original generic response tree without cloning secret bytes.
    #[must_use]
    pub fn into_response_message(self) -> Structure {
        self.response_message
    }
}

/// A payload-free error from parsing the test-only OASIS fixture format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OasisCryptoFixtureError {
    /// The XML document is malformed.
    MalformedXml,
    /// The document exceeds local parsing limits or contains a DTD.
    XmlLimitOrDtd,
    /// The document does not have the expected fixture/message shape.
    InvalidFixtureShape,
    /// Request and response message counts do not match.
    UnpairedMessages,
    /// A selected request and response identify different operations.
    MismatchedOperations,
    /// A selected request repeats a source sequence.
    DuplicateSequence,
    /// A selected message contains a symbol outside the supported set.
    UnknownSymbol,
    /// A selected message contains an unknown or invalid tag.
    InvalidTag,
    /// A selected message uses a TTLV Item Type this adapter does not support.
    UnsupportedItemType,
    /// A selected message contains a value not representable by the TTLV model.
    InvalidValue,
    /// A selected message exceeds TTLV tree limits.
    TtlvLimit,
    /// The checked-in normative catalog is malformed or ambiguous.
    InvalidCatalog,
}

impl fmt::Display for OasisCryptoFixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MalformedXml => "OASIS fixture XML is malformed",
            Self::XmlLimitOrDtd => "OASIS fixture XML exceeds limits or contains a DTD",
            Self::InvalidFixtureShape => "OASIS fixture message shape is invalid",
            Self::UnpairedMessages => "OASIS fixture request and response messages are unpaired",
            Self::MismatchedOperations => "OASIS fixture request and response operations differ",
            Self::DuplicateSequence => "OASIS fixture contains a duplicate operation sequence",
            Self::UnknownSymbol => "OASIS fixture selected message contains an unsupported symbol",
            Self::InvalidTag => "OASIS fixture selected message contains an invalid tag",
            Self::UnsupportedItemType => {
                "OASIS fixture selected message contains an unsupported TTLV Item Type"
            }
            Self::InvalidValue => "OASIS fixture selected message contains an invalid TTLV value",
            Self::TtlvLimit => "OASIS fixture selected message exceeds TTLV model limits",
            Self::InvalidCatalog => "KMIPKit normative catalog cannot resolve fixture values",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for OasisCryptoFixtureError {}

fn contains_doctype_declaration(xml: &str) -> bool {
    const DOCTYPE: &[u8] = b"<!DOCTYPE";
    xml.as_bytes()
        .windows(DOCTYPE.len())
        .any(|window| window.eq_ignore_ascii_case(DOCTYPE))
}

#[derive(Clone)]
struct RequestMetadata<'a, 'input> {
    node: Node<'a, 'input>,
    operation: Option<OasisCryptoOperation>,
    source_sequence: Option<usize>,
    step_identity: Option<String>,
}

#[derive(Clone, Copy)]
struct ResponseMetadata<'a, 'input> {
    node: Node<'a, 'input>,
    operation: Option<OasisCryptoOperation>,
}

fn request_metadata<'a, 'input>(
    node: Node<'a, 'input>,
) -> Result<RequestMetadata<'a, 'input>, OasisCryptoFixtureError> {
    let batch = unique_child(node, "BatchItem")?;
    let operation = operation_kind(batch)?;
    let (source_sequence, step_identity) = if operation.is_some() {
        let header = unique_child(node, "RequestHeader")?;
        let identity = value_attribute(unique_child(header, "ClientCorrelationValue")?)?;
        let sequence = parse_step_sequence(identity)?;
        (Some(sequence), Some(identity.to_owned()))
    } else {
        (None, None)
    };

    Ok(RequestMetadata {
        node,
        operation,
        source_sequence,
        step_identity,
    })
}

fn response_metadata<'a, 'input>(
    node: Node<'a, 'input>,
) -> Result<ResponseMetadata<'a, 'input>, OasisCryptoFixtureError> {
    let batch = unique_child(node, "BatchItem")?;
    Ok(ResponseMetadata {
        node,
        operation: operation_kind(batch)?,
    })
}

fn operation_kind(
    batch: Node<'_, '_>,
) -> Result<Option<OasisCryptoOperation>, OasisCryptoFixtureError> {
    let operation = unique_child(batch, "Operation")?;
    if operation.attribute("type") != Some("Enumeration") {
        return Err(OasisCryptoFixtureError::InvalidFixtureShape);
    }
    Ok(match value_attribute(operation)? {
        "Encrypt" => Some(OasisCryptoOperation::Encrypt),
        "Decrypt" => Some(OasisCryptoOperation::Decrypt),
        numeric => match parse_unsigned(numeric).ok() {
            Some(ENCRYPT_OPERATION) => Some(OasisCryptoOperation::Encrypt),
            Some(DECRYPT_OPERATION) => Some(OasisCryptoOperation::Decrypt),
            _ => None,
        },
    })
}

fn parse_step_sequence(identity: &str) -> Result<usize, OasisCryptoFixtureError> {
    let (_, step) = identity
        .rsplit_once(" step=")
        .ok_or(OasisCryptoFixtureError::InvalidFixtureShape)?;
    if step.is_empty() || !step.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(OasisCryptoFixtureError::InvalidFixtureShape);
    }
    step.parse()
        .map_err(|_| OasisCryptoFixtureError::InvalidFixtureShape)
}

fn parse_message(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
) -> Result<Structure, OasisCryptoFixtureError> {
    let mut message = Structure::new();
    let mut item_count = 0;
    for child in node.children().filter(Node::is_element) {
        message
            .try_push(parse_item(child, catalog, 1, &mut item_count)?)
            .map_err(|_| OasisCryptoFixtureError::TtlvLimit)?;
    }
    Ok(message)
}

fn parse_item(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
    depth: usize,
    item_count: &mut usize,
) -> Result<Item, OasisCryptoFixtureError> {
    *item_count = item_count
        .checked_add(1)
        .ok_or(OasisCryptoFixtureError::TtlvLimit)?;
    if *item_count > MAX_TTLV_ITEMS || depth > MAX_TTLV_DEPTH {
        return Err(OasisCryptoFixtureError::TtlvLimit);
    }

    let tag = catalog.tag_for(node)?;
    // OASIS test-case XML omits `type="Structure"` on elements that contain
    // child Items. A leaf without an explicit type is not unambiguously typed.
    let item_type = node
        .attribute("type")
        .or_else(|| {
            node.children()
                .any(|child| child.node_type() == NodeType::Element)
                .then_some("Structure")
        })
        .ok_or(OasisCryptoFixtureError::InvalidValue)?;
    let value = match item_type {
        "Structure" => {
            if node.attribute("value").is_some() {
                return Err(OasisCryptoFixtureError::InvalidValue);
            }
            let mut structure = Structure::new();
            for child in node.children().filter(Node::is_element) {
                structure
                    .try_push(parse_item(child, catalog, depth + 1, item_count)?)
                    .map_err(|_| OasisCryptoFixtureError::TtlvLimit)?;
            }
            Value::structure(structure)
        }
        "Integer" => Value::integer(parse_scalar(node)?),
        "LongInteger" => Value::long_integer(parse_scalar(node)?),
        "BigInteger" => Value::big_integer(parse_byte_string(node, "BigInteger")?),
        "Enumeration" => Value::enumeration(catalog.enumeration_value(node)?),
        "Boolean" => Value::boolean(parse_boolean(node)?),
        "TextString" => Value::text_string(parse_text_string(node)?),
        "ByteString" => Value::byte_string(parse_byte_string(node, "ByteString")?),
        "DateTime" => Value::date_time(parse_date_time(node)?),
        "Interval" => Value::interval(parse_scalar(node)?),
        "DateTimeExtended" => Value::date_time_extended(parse_scalar(node)?),
        _ => return Err(OasisCryptoFixtureError::UnsupportedItemType),
    };

    let checked_tag = RawTag::new(tag)
        .and_then(|raw| raw.try_checked())
        .map_err(|_| OasisCryptoFixtureError::InvalidTag)?;
    Item::new(checked_tag, value).map_err(|_| OasisCryptoFixtureError::InvalidValue)
}

fn parse_scalar<T>(node: Node<'_, '_>) -> Result<T, OasisCryptoFixtureError>
where
    T: std::str::FromStr,
{
    let value = value_attribute(node)?;
    reject_symbol(value)?;
    if node
        .children()
        .any(|child| child.node_type() == NodeType::Element)
    {
        return Err(OasisCryptoFixtureError::InvalidValue);
    }
    value
        .parse()
        .map_err(|_| OasisCryptoFixtureError::InvalidValue)
}

fn parse_boolean(node: Node<'_, '_>) -> Result<bool, OasisCryptoFixtureError> {
    let value = value_attribute(node)?;
    reject_symbol(value)?;
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(OasisCryptoFixtureError::InvalidValue),
    }
}

fn parse_text_string(node: Node<'_, '_>) -> Result<String, OasisCryptoFixtureError> {
    let value = value_attribute(node)?;
    if value == "$UNIQUE_IDENTIFIER_0" {
        return Ok(TEST_UNIQUE_IDENTIFIER_0.to_owned());
    }
    reject_symbol(value)?;
    Ok(value.to_owned())
}

fn parse_date_time(node: Node<'_, '_>) -> Result<i64, OasisCryptoFixtureError> {
    let value = value_attribute(node)?;
    if value == "$NOW" {
        return Ok(TEST_NOW);
    }
    reject_symbol(value)?;
    value
        .parse()
        .map_err(|_| OasisCryptoFixtureError::InvalidValue)
}

fn parse_byte_string(
    node: Node<'_, '_>,
    item_type: &str,
) -> Result<Vec<u8>, OasisCryptoFixtureError> {
    let value = value_attribute(node)?;
    if item_type == "ByteString" && value == "$CORRELATION_VALUE" {
        return Ok(TEST_CORRELATION_VALUE.to_vec());
    }
    reject_symbol(value)?;
    if value.len() % 2 != 0 {
        return Err(OasisCryptoFixtureError::InvalidValue);
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits =
                std::str::from_utf8(pair).map_err(|_| OasisCryptoFixtureError::InvalidValue)?;
            u8::from_str_radix(digits, 16).map_err(|_| OasisCryptoFixtureError::InvalidValue)
        })
        .collect()
}

fn reject_symbol(value: &str) -> Result<(), OasisCryptoFixtureError> {
    if value.contains('$') {
        Err(OasisCryptoFixtureError::UnknownSymbol)
    } else {
        Ok(())
    }
}

fn value_attribute<'a>(node: Node<'a, 'a>) -> Result<&'a str, OasisCryptoFixtureError> {
    node.attribute("value")
        .ok_or(OasisCryptoFixtureError::InvalidValue)
}

fn unique_child<'a, 'input>(
    parent: Node<'a, 'input>,
    name: &str,
) -> Result<Node<'a, 'input>, OasisCryptoFixtureError> {
    let mut children = parent
        .children()
        .filter(Node::is_element)
        .filter(|child| child.tag_name().name() == name);
    let child = children
        .next()
        .ok_or(OasisCryptoFixtureError::InvalidFixtureShape)?;
    if children.next().is_some() {
        return Err(OasisCryptoFixtureError::InvalidFixtureShape);
    }
    Ok(child)
}

struct CatalogMaps {
    tags: HashMap<String, u32>,
    enumerations: HashMap<String, HashMap<String, Option<u32>>>,
}

impl CatalogMaps {
    fn load() -> Result<Self, OasisCryptoFixtureError> {
        let catalog: JsonValue = serde_json::from_str(NORMATIVE_CATALOG)
            .map_err(|_| OasisCryptoFixtureError::InvalidCatalog)?;
        let elements = catalog
            .get("elements")
            .and_then(JsonValue::as_array)
            .ok_or(OasisCryptoFixtureError::InvalidCatalog)?;
        let mut enumeration_names = HashMap::new();
        for element in elements {
            if string_field(element, "kind") == Some("enumeration")
                && let (Some(id), Some(name)) = (
                    string_field(element, "element_id"),
                    string_field(element, "name"),
                )
            {
                enumeration_names.insert(id.to_owned(), normalize_name(name));
            }
        }

        let mut tags = HashMap::new();
        let mut enumerations: HashMap<String, HashMap<String, Option<u32>>> = HashMap::new();
        for element in elements {
            match string_field(element, "kind") {
                Some("tag") => {
                    if matches!(
                        string_field(element, "allocation"),
                        Some("reserved" | "unused")
                    ) {
                        continue;
                    }
                    let (Some(name), Some(wire_value)) = (
                        string_field(element, "name"),
                        string_field(element, "wire_value"),
                    ) else {
                        return Err(OasisCryptoFixtureError::InvalidCatalog);
                    };
                    let key = normalize_name(name);
                    let raw = parse_catalog_hex(wire_value)?;
                    if tags
                        .insert(key.clone(), raw)
                        .is_some_and(|previous| previous != raw)
                    {
                        return Err(OasisCryptoFixtureError::InvalidCatalog);
                    }
                }
                Some("enumeration_value") => {
                    // OASIS extension ranges use symbolic patterns such as
                    // `8XXXXXXX`, not concrete values that can be encoded.
                    // Preserve extension values only when the fixture gives
                    // their numeric wire value explicitly.
                    if string_field(element, "allocation") == Some("extension") {
                        continue;
                    }
                    let (Some(name), Some(wire_value)) = (
                        string_field(element, "name"),
                        string_field(element, "wire_value"),
                    ) else {
                        return Err(OasisCryptoFixtureError::InvalidCatalog);
                    };
                    // Reserved ranges can also be represented symbolically
                    // (for example `00000001-00000103`). They are not a
                    // concrete value and must not be guessed by this adapter.
                    if !wire_value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                        continue;
                    }
                    let parents = element
                        .get("parent_element_ids")
                        .and_then(JsonValue::as_array)
                        .ok_or(OasisCryptoFixtureError::InvalidCatalog)?;
                    for parent in parents.iter().filter_map(JsonValue::as_str) {
                        let Some(category) = enumeration_names.get(parent) else {
                            continue;
                        };
                        let raw = parse_catalog_hex(wire_value)?;
                        let values = enumerations.entry(category.clone()).or_default();
                        let key = normalize_name(name);
                        match values.entry(key) {
                            Entry::Vacant(entry) => {
                                entry.insert(Some(raw));
                            }
                            Entry::Occupied(mut entry) => {
                                if let Some(previous) = *entry.get()
                                    && previous != raw
                                {
                                    entry.insert(None);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self { tags, enumerations })
    }

    fn tag_for(&self, node: Node<'_, '_>) -> Result<u32, OasisCryptoFixtureError> {
        let name = normalize_name(node.tag_name().name());
        match (self.tags.get(&name), node.attribute("tag")) {
            (Some(raw), None) => Ok(*raw),
            (Some(raw), Some(explicit)) if parse_unsigned(explicit).ok() == Some(*raw) => Ok(*raw),
            (Some(_), Some(_)) | (None, None) => Err(OasisCryptoFixtureError::InvalidTag),
            (None, Some(explicit)) => {
                parse_unsigned(explicit).map_err(|_| OasisCryptoFixtureError::InvalidTag)
            }
        }
    }

    fn enumeration_value(&self, node: Node<'_, '_>) -> Result<u32, OasisCryptoFixtureError> {
        let value = value_attribute(node)?;
        if value.starts_with("0x") || value.bytes().all(|byte| byte.is_ascii_digit()) {
            return parse_unsigned(value);
        }
        reject_symbol(value)?;
        let category = normalize_name(node.tag_name().name());
        self.enumerations
            .get(&category)
            .and_then(|values| values.get(&normalize_name(value)))
            .copied()
            .flatten()
            .ok_or(OasisCryptoFixtureError::InvalidValue)
    }
}

fn string_field<'a>(value: &'a JsonValue, key: &str) -> Option<&'a str> {
    value.get(key).and_then(JsonValue::as_str)
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn parse_catalog_hex(value: &str) -> Result<u32, OasisCryptoFixtureError> {
    u32::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16)
        .map_err(|_| OasisCryptoFixtureError::InvalidCatalog)
}

fn parse_unsigned(value: &str) -> Result<u32, OasisCryptoFixtureError> {
    if let Some(hex) = value.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|_| OasisCryptoFixtureError::InvalidValue)
    } else {
        value
            .parse()
            .map_err(|_| OasisCryptoFixtureError::InvalidValue)
    }
}
