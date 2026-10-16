//! Test-only adapter for the pinned OASIS Encrypt and Decrypt XML fixtures.
//!
//! The adapter retains each selected message as an ordered generic TTLV tree.
//! It substitutes only the documented fixture symbols after filtering to
//! Encrypt and Decrypt items. It does not validate complete Test Case flows.

use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::fmt;

use kmipkit_ttlv::{Item, RawTag, Structure, Value};
use roxmltree::{Document, Error as XmlError, Node, NodeType, ParsingOptions};
use serde_json::Value as JsonValue;

const NORMATIVE_CATALOG: &str = include_str!("../../../specification/catalog/kmip-2.1.json");
const MAX_XML_BYTES: usize = 16 * 1024 * 1024;
// Leaves room above the 100,000 TTLV-item limit for message envelopes,
// formatting whitespace, and other XML nodes before TTLV conversion.
const MAX_XML_NODES: u32 = 250_000;
const MAX_TTLV_DEPTH: usize = 64;
const MAX_TTLV_ITEMS: usize = 100_000;
const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const TEST_NOW: i64 = 1_700_000_000;
const TEST_UNIQUE_IDENTIFIER_0: &str = "kmipkit-test-unique-id-0";
const TEST_CORRELATION_VALUE: &[u8] = b"kmipkit-test-correlation-value-0";
const SUPPORTED_PROFILE_OPERATIONS: &[u32] = &[
    0x0000_0001, // Create
    0x0000_0003, // Register
    0x0000_0009, // Check
    0x0000_000b, // Get Attributes
    0x0000_000a, // Get
    0x0000_000d, // Add Attribute
    0x0000_000e, // Modify Attribute
    0x0000_0012, // Activate
    0x0000_0013, // Revoke
    0x0000_0014, // Destroy
    0x0000_0008, // Locate
    0x0000_002c, // Log
    0x0000_0034, // Interop
];
const CRYPTOGRAPHIC_USAGE_MASKS: &[(&str, i32)] = &[
    ("Encrypt", 0x0000_0004),
    ("Decrypt", 0x0000_0008),
    ("Sign", 0x0000_0001),
    ("Verify", 0x0000_0002),
    ("WrapKey", 0x0000_0010),
];

#[cfg(test)]
#[path = "../tests/fixtures/oasis_get_fixture_tests.rs"]
mod managed_object_fixture_tests;

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

/// An ordered request/response stream from one pinned KMIP profile XML case.
///
/// This is fixture replay input only. It does not assert profile conformance or
/// server interoperability.
#[derive(Debug)]
pub struct OasisProfileFixture {
    case_id: String,
    exchanges: Vec<OasisProfileExchange>,
}

/// One source-ordered exchange in an [`OasisProfileFixture`].
#[derive(Debug)]
pub struct OasisProfileExchange {
    step: usize,
    operations: Vec<u32>,
    request_message: Structure,
    response_message: Structure,
}

/// Deterministic substitutions supported by the pinned profile fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OasisProfileFixtureBindings {
    /// Value substituted for the fixture's `$NOW` symbol.
    pub now: i64,
    /// Value substituted for the fixture's `$UNIQUE_IDENTIFIER_0` symbol.
    pub unique_identifier_0: &'static str,
}

impl Default for OasisProfileFixtureBindings {
    fn default() -> Self {
        Self {
            now: TEST_NOW,
            unique_identifier_0: TEST_UNIQUE_IDENTIFIER_0,
        }
    }
}

impl OasisProfileFixture {
    /// Parses one complete profile fixture as strict ordered exchanges.
    ///
    /// This API preserves every paired operation and never filters or
    /// reconstructs missing messages. It supports only the two documented
    /// fixture substitutions in [`OasisProfileFixtureBindings`].
    ///
    /// # Errors
    ///
    /// Returns a payload-free error when the XML shape, message pairing,
    /// operation, supported symbol, TTLV value, or resource limit is invalid.
    pub fn from_xml(
        case_id: &str,
        xml: &str,
        bindings: OasisProfileFixtureBindings,
    ) -> Result<Self, OasisCryptoFixtureError> {
        if case_id.is_empty() || xml.len() > MAX_XML_BYTES || contains_doctype_declaration(xml) {
            return Err(OasisCryptoFixtureError::XmlLimitOrDtd);
        }

        let document = Document::parse_with_options(
            xml,
            ParsingOptions {
                nodes_limit: MAX_XML_NODES,
                ..ParsingOptions::default()
            },
        )
        .map_err(|error| match error {
            XmlError::NodesLimitReached => OasisCryptoFixtureError::XmlNodeLimit,
            _ => OasisCryptoFixtureError::MalformedXml,
        })?;
        let root = document.root_element();
        if root.tag_name().name() != "KMIP" {
            return Err(OasisCryptoFixtureError::InvalidFixtureShape);
        }

        let messages: Vec<_> = root.children().filter(Node::is_element).collect();
        if messages.is_empty() || messages.len() % 2 != 0 {
            return Err(OasisCryptoFixtureError::UnpairedMessages);
        }
        let catalog = CatalogMaps::load()?;
        let mut exchanges = Vec::with_capacity(messages.len() / 2);
        let (message_pairs, remaining) = messages.as_slice().as_chunks::<2>();
        if !remaining.is_empty() {
            return Err(OasisCryptoFixtureError::UnpairedMessages);
        }
        for (step, [request_node, response_node]) in message_pairs.iter().enumerate() {
            if request_node.tag_name().name() != "RequestMessage"
                || response_node.tag_name().name() != "ResponseMessage"
            {
                return Err(OasisCryptoFixtureError::InvalidFixtureShape);
            }

            let request_batches = validate_batch_message(*request_node, "RequestHeader", false)?;
            let response_batches = validate_batch_message(*response_node, "ResponseHeader", true)?;
            if request_batches.len() != response_batches.len() {
                return Err(OasisCryptoFixtureError::UnpairedMessages);
            }
            let mut operations = Vec::with_capacity(request_batches.len());
            for (request_batch, response_batch) in request_batches.iter().zip(&response_batches) {
                let request_operation = profile_operation(&catalog, *request_batch)?;
                let response_operation = profile_operation(&catalog, *response_batch)?;
                if request_operation != response_operation {
                    return Err(OasisCryptoFixtureError::MismatchedOperations);
                }
                validate_batch_item_pair(*request_batch, *response_batch)?;
                operations.push(request_operation);
            }
            validate_profile_step(case_id, step, *request_node)?;

            let request_message = parse_profile_message(*request_node, &catalog, &bindings)?;
            let response_message = parse_profile_message(*response_node, &catalog, &bindings)?;
            exchanges.push(OasisProfileExchange {
                step,
                operations,
                request_message,
                response_message,
            });
        }

        Ok(Self {
            case_id: case_id.to_owned(),
            exchanges,
        })
    }

    /// Returns the exact caller-supplied official case identifier.
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns every exchange in source order.
    #[must_use]
    pub fn exchanges(&self) -> &[OasisProfileExchange] {
        &self.exchanges
    }
}

impl OasisProfileExchange {
    /// Zero-based source position of this request/response pair.
    #[must_use]
    pub const fn step(&self) -> usize {
        self.step
    }

    /// Raw KMIP Operation Enumerations in the fixture's batch order.
    #[must_use]
    pub fn operations(&self) -> &[u32] {
        &self.operations
    }

    /// Complete generic TTLV request message parsed from the fixture.
    #[must_use]
    pub const fn request_message(&self) -> &Structure {
        &self.request_message
    }

    /// Complete generic TTLV response message parsed from the fixture.
    #[must_use]
    pub const fn response_message(&self) -> &Structure {
        &self.response_message
    }
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

        let document = Document::parse_with_options(
            xml,
            ParsingOptions {
                nodes_limit: MAX_XML_NODES,
                ..ParsingOptions::default()
            },
        )
        .map_err(|error| match error {
            XmlError::NodesLimitReached => OasisCryptoFixtureError::XmlNodeLimit,
            _ => OasisCryptoFixtureError::MalformedXml,
        })?;
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
    /// The XML document exceeds the parser node limit.
    XmlNodeLimit,
    /// The document does not have the expected fixture/message shape.
    InvalidFixtureShape,
    /// Request and response message counts do not match.
    UnpairedMessages,
    /// A selected request and response identify different operations.
    MismatchedOperations,
    /// Paired batch items have different Unique Batch Item IDs.
    MismatchedBatchItemIdentifiers,
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
    /// The profile fixture contains an operation outside this replay runner.
    UnsupportedOperation,
    /// The profile fixture contains a valid but unsupported wrapper construct.
    UnsupportedFixtureConstruct,
    /// Explicit source step identifiers are absent, duplicated, or reordered.
    ReorderedSteps,
}

impl fmt::Display for OasisCryptoFixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MalformedXml => "OASIS fixture XML is malformed",
            Self::XmlLimitOrDtd => "OASIS fixture XML exceeds limits or contains a DTD",
            Self::XmlNodeLimit => "OASIS fixture XML exceeds the configured node limit",
            Self::InvalidFixtureShape => "OASIS fixture message shape is invalid",
            Self::UnpairedMessages => "OASIS fixture request and response messages are unpaired",
            Self::MismatchedOperations => "OASIS fixture request and response operations differ",
            Self::MismatchedBatchItemIdentifiers => {
                "OASIS profile fixture request and response batch IDs differ"
            }
            Self::DuplicateSequence => "OASIS fixture contains a duplicate operation sequence",
            Self::UnknownSymbol => "OASIS fixture selected message contains an unsupported symbol",
            Self::InvalidTag => "OASIS fixture selected message contains an invalid tag",
            Self::UnsupportedItemType => {
                "OASIS fixture selected message contains an unsupported TTLV Item Type"
            }
            Self::InvalidValue => "OASIS fixture selected message contains an invalid TTLV value",
            Self::TtlvLimit => "OASIS fixture selected message exceeds TTLV model limits",
            Self::InvalidCatalog => "KMIPKit normative catalog cannot resolve fixture values",
            Self::UnsupportedOperation => "OASIS profile fixture operation is unsupported",
            Self::UnsupportedFixtureConstruct => "OASIS profile fixture construct is unsupported",
            Self::ReorderedSteps => "OASIS profile fixture source steps are reordered",
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

fn profile_operation(
    catalog: &CatalogMaps,
    batch: Node<'_, '_>,
) -> Result<u32, OasisCryptoFixtureError> {
    let value = catalog.enumeration_value(unique_child(batch, "Operation")?)?;
    if SUPPORTED_PROFILE_OPERATIONS.contains(&value) {
        Ok(value)
    } else {
        Err(OasisCryptoFixtureError::UnsupportedOperation)
    }
}

fn validate_batch_message<'a, 'input>(
    message: Node<'a, 'input>,
    header_name: &str,
    is_response: bool,
) -> Result<Vec<Node<'a, 'input>>, OasisCryptoFixtureError> {
    let names: Vec<_> = message
        .children()
        .filter(Node::is_element)
        .map(|child| child.tag_name().name())
        .collect();
    if names.len() < 2
        || names[0] != header_name
        || names[1..].iter().any(|name| *name != "BatchItem")
    {
        return Err(OasisCryptoFixtureError::UnsupportedFixtureConstruct);
    }
    let header = unique_child(message, header_name)?;
    let count = unique_child(header, "BatchCount")?;
    let batch_count = if count.attribute("type") == Some("Integer") {
        parse_scalar::<usize>(count)?
    } else {
        return Err(OasisCryptoFixtureError::UnsupportedFixtureConstruct);
    };
    let batches: Vec<_> = message
        .children()
        .filter(Node::is_element)
        .filter(|child| child.tag_name().name() == "BatchItem")
        .collect();
    if batch_count == 0 || batches.len() != batch_count {
        return Err(OasisCryptoFixtureError::UnsupportedFixtureConstruct);
    }
    for batch in &batches {
        validate_batch_item_shape(*batch, is_response)?;
    }
    Ok(batches)
}

fn validate_batch_item_shape(
    batch: Node<'_, '_>,
    is_response: bool,
) -> Result<(), OasisCryptoFixtureError> {
    let batch_names: Vec<_> = batch
        .children()
        .filter(Node::is_element)
        .map(|child| child.tag_name().name())
        .collect();
    let allowed: &[&[&str]] = if is_response {
        &[
            &["Operation", "ResultStatus", "ResponsePayload"],
            &["Operation", "ResultStatus", "ResultReason", "ResultMessage"],
            &[
                "Operation",
                "UniqueBatchItemID",
                "ResultStatus",
                "ResponsePayload",
            ],
            &[
                "Operation",
                "UniqueBatchItemID",
                "ResultStatus",
                "ResultReason",
                "ResultMessage",
            ],
        ]
    } else {
        &[
            &["Operation", "RequestPayload"],
            &["Operation", "UniqueBatchItemID", "RequestPayload"],
        ]
    };
    if !allowed.contains(&batch_names.as_slice()) {
        return Err(OasisCryptoFixtureError::UnsupportedFixtureConstruct);
    }
    Ok(())
}

fn validate_batch_item_pair(
    request: Node<'_, '_>,
    response: Node<'_, '_>,
) -> Result<(), OasisCryptoFixtureError> {
    let request_id = optional_child(request, "UniqueBatchItemID")?;
    let response_id = optional_child(response, "UniqueBatchItemID")?;
    match (request_id, response_id) {
        (None, None) => Ok(()),
        (Some(request_id), Some(response_id))
            if request_id.attribute("type") == Some("ByteString")
                && response_id.attribute("type") == Some("ByteString")
                && request_id.attribute("value") == response_id.attribute("value") =>
        {
            Ok(())
        }
        _ => Err(OasisCryptoFixtureError::MismatchedBatchItemIdentifiers),
    }
}

fn optional_child<'a, 'input>(
    parent: Node<'a, 'input>,
    name: &str,
) -> Result<Option<Node<'a, 'input>>, OasisCryptoFixtureError> {
    let mut children = parent
        .children()
        .filter(Node::is_element)
        .filter(|child| child.tag_name().name() == name);
    let child = children.next();
    if children.next().is_some() {
        return Err(OasisCryptoFixtureError::InvalidFixtureShape);
    }
    Ok(child)
}

fn validate_profile_step(
    case_id: &str,
    step: usize,
    request: Node<'_, '_>,
) -> Result<(), OasisCryptoFixtureError> {
    let header = unique_child(request, "RequestHeader")?;
    let identity = header
        .children()
        .filter(Node::is_element)
        .find(|child| child.tag_name().name() == "ClientCorrelationValue");
    if let Some(identity) = identity {
        let value = value_attribute(identity)?;
        if value.contains(" step=") {
            let parsed = parse_step_sequence(value)?;
            if parsed != step
                || value.rsplit_once(" step=").map(|(prefix, _)| prefix) != Some(case_id)
            {
                return Err(OasisCryptoFixtureError::ReorderedSteps);
            }
        }
    }
    Ok(())
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
    parse_message_with_get_test_symbols(node, catalog, false)
}

#[cfg(test)]
fn parse_get_message_with_test_symbols(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
) -> Result<Structure, OasisCryptoFixtureError> {
    parse_message_with_get_test_symbols(node, catalog, true)
}

fn parse_message_with_get_test_symbols(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
    allow_get_test_symbols: bool,
) -> Result<Structure, OasisCryptoFixtureError> {
    parse_message_using_bindings(
        node,
        catalog,
        OasisProfileFixtureBindings::default(),
        allow_get_test_symbols,
        true,
    )
}

fn parse_profile_message(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
    bindings: &OasisProfileFixtureBindings,
) -> Result<Structure, OasisCryptoFixtureError> {
    parse_message_using_bindings(node, catalog, *bindings, false, false)
}

fn parse_message_using_bindings(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
    bindings: OasisProfileFixtureBindings,
    allow_get_test_symbols: bool,
    allow_correlation_value: bool,
) -> Result<Structure, OasisCryptoFixtureError> {
    let mut message = Structure::new();
    let mut item_count = 0;
    for child in node.children().filter(Node::is_element) {
        message
            .try_push(parse_item_with_get_test_symbols(
                child,
                catalog,
                1,
                &mut item_count,
                allow_get_test_symbols,
                bindings,
                allow_correlation_value,
            )?)
            .map_err(|_| OasisCryptoFixtureError::TtlvLimit)?;
    }
    Ok(message)
}

fn parse_item_with_get_test_symbols(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
    depth: usize,
    item_count: &mut usize,
    allow_get_test_symbols: bool,
    bindings: OasisProfileFixtureBindings,
    allow_correlation_value: bool,
) -> Result<Item, OasisCryptoFixtureError> {
    parse_item_inner(
        node,
        catalog,
        depth,
        item_count,
        allow_get_test_symbols,
        bindings,
        allow_correlation_value,
    )
}

fn parse_item_inner(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
    depth: usize,
    item_count: &mut usize,
    allow_get_test_symbols: bool,
    bindings: OasisProfileFixtureBindings,
    allow_correlation_value: bool,
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
        .or_else(|| (node.tag_name().name() == "ResponsePayload").then_some("Structure"))
        .ok_or(OasisCryptoFixtureError::InvalidValue)?;
    let has_element_children = node
        .children()
        .any(|child| child.node_type() == NodeType::Element);
    if item_type != "Structure" && has_element_children {
        return Err(OasisCryptoFixtureError::InvalidValue);
    }
    let value = match item_type {
        "Structure" => {
            if node.attribute("value").is_some() {
                return Err(OasisCryptoFixtureError::InvalidValue);
            }
            let mut structure = Structure::new();
            for child in node.children().filter(Node::is_element) {
                structure
                    .try_push(parse_item_with_get_test_symbols(
                        child,
                        catalog,
                        depth + 1,
                        item_count,
                        allow_get_test_symbols,
                        bindings,
                        allow_correlation_value,
                    )?)
                    .map_err(|_| OasisCryptoFixtureError::TtlvLimit)?;
            }
            Value::structure(structure)
        }
        "Integer" => Value::integer(parse_integer(node)?),
        "LongInteger" => Value::long_integer(parse_scalar(node)?),
        // Test-only symbolic substitutions apply only to ByteString leaves.
        "BigInteger" => Value::big_integer(parse_byte_string(
            node,
            "BigInteger",
            false,
            allow_correlation_value,
        )?),
        "Enumeration" => Value::enumeration(parse_enumeration(node, catalog)?),
        "Boolean" => Value::boolean(parse_boolean(node)?),
        "TextString" => {
            Value::text_string(parse_text_string(node, allow_get_test_symbols, bindings)?)
        }
        "ByteString" => Value::byte_string(parse_byte_string(
            node,
            "ByteString",
            allow_get_test_symbols,
            allow_correlation_value,
        )?),
        "DateTime" => Value::date_time(parse_date_time(node, bindings)?),
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
    value
        .parse()
        .map_err(|_| OasisCryptoFixtureError::InvalidValue)
}

fn parse_integer(node: Node<'_, '_>) -> Result<i32, OasisCryptoFixtureError> {
    let value = value_attribute(node)?;
    if normalize_name(node.tag_name().name()) != "cryptographicusagemask" {
        reject_symbol(value)?;
        return value
            .parse()
            .map_err(|_| OasisCryptoFixtureError::InvalidValue);
    }
    let mut mask = 0;
    let mut seen = HashSet::new();
    for part in value.split_ascii_whitespace() {
        reject_symbol(part)?;
        let Some((_, bit)) = CRYPTOGRAPHIC_USAGE_MASKS
            .iter()
            .find(|(name, _)| *name == part)
        else {
            return Err(OasisCryptoFixtureError::InvalidValue);
        };
        if !seen.insert(part) {
            return Err(OasisCryptoFixtureError::InvalidValue);
        }
        mask |= bit;
    }
    if seen.is_empty() {
        return Err(OasisCryptoFixtureError::InvalidValue);
    }
    Ok(mask)
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

fn parse_enumeration(
    node: Node<'_, '_>,
    catalog: &CatalogMaps,
) -> Result<u32, OasisCryptoFixtureError> {
    if normalize_name(node.tag_name().name()) == "attributereference" {
        let value = value_attribute(node)?;
        if value.starts_with("0x") || value.bytes().all(|byte| byte.is_ascii_digit()) {
            return parse_unsigned(value);
        }
        reject_symbol(value)?;
        return catalog
            .tags
            .get(&normalize_name(value))
            .copied()
            .ok_or(OasisCryptoFixtureError::InvalidValue);
    }
    catalog.enumeration_value(node)
}

fn parse_text_string(
    node: Node<'_, '_>,
    allow_get_test_symbols: bool,
    bindings: OasisProfileFixtureBindings,
) -> Result<String, OasisCryptoFixtureError> {
    #[cfg(not(test))]
    let _ = allow_get_test_symbols;
    let value = value_attribute(node)?;
    if value == "$UNIQUE_IDENTIFIER_0" {
        return Ok(bindings.unique_identifier_0.to_owned());
    }
    #[cfg(test)]
    if allow_get_test_symbols
        && normalize_name(node.tag_name().name()) == "uniqueidentifier"
        && let Some(index) = value.strip_prefix("$UNIQUE_IDENTIFIER_")
        && matches!(index, "1" | "2")
    {
        return Ok(format!("kmipkit-test-unique-id-{index}"));
    }
    reject_symbol(value)?;
    Ok(value.to_owned())
}

fn parse_date_time(
    node: Node<'_, '_>,
    bindings: OasisProfileFixtureBindings,
) -> Result<i64, OasisCryptoFixtureError> {
    let value = value_attribute(node)?;
    if value == "$NOW" {
        return Ok(bindings.now);
    }
    reject_symbol(value)?;
    match value.parse::<i64>() {
        Ok(seconds) => Ok(seconds),
        Err(_) => {
            parse_rfc3339_date_time(value).map_err(|()| OasisCryptoFixtureError::InvalidValue)
        }
    }
}

fn parse_rfc3339_date_time(value: &str) -> Result<i64, ()> {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return Err(());
    }
    let parse_two = |start: usize| -> Result<i32, ()> {
        let digits = bytes.get(start..start + 2).ok_or(())?;
        if !digits.iter().all(u8::is_ascii_digit) {
            return Err(());
        }
        let text = std::str::from_utf8(digits).map_err(|_| ())?;
        text.parse().map_err(|_| ())
    };
    let year_text = std::str::from_utf8(bytes.get(..4).ok_or(())?).map_err(|_| ())?;
    if !year_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    let year: i32 = year_text.parse().map_err(|_| ())?;
    let month = parse_two(5)?;
    let day = parse_two(8)?;
    let hour = parse_two(11)?;
    let minute = parse_two(14)?;
    let second = parse_two(17)?;
    if !(1..=12).contains(&month) || !(0..=23).contains(&hour) || minute > 59 || second > 59 {
        return Err(());
    }
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        2 if leap_year => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days_in_month).contains(&day) {
        return Err(());
    }
    let timezone_offset = match bytes.get(19).copied() {
        Some(b'Z') if bytes.len() == 20 => 0_i64,
        Some(sign @ (b'+' | b'-')) if bytes.len() == 25 && bytes[22] == b':' => {
            let offset_hour = parse_two(20)?;
            let offset_minute = parse_two(23)?;
            if offset_hour > 23 || offset_minute > 59 {
                return Err(());
            }
            let offset = i64::from(offset_hour) * 3_600 + i64::from(offset_minute) * 60;
            if sign == b'+' { offset } else { -offset }
        }
        _ => return Err(()),
    };

    let mut days = 0_i64;
    if year >= 1970 {
        for prior_year in 1970..year {
            days += if is_leap_year(prior_year) { 366 } else { 365 };
        }
    } else {
        for prior_year in year..1970 {
            days -= if is_leap_year(prior_year) { 366 } else { 365 };
        }
    }
    for prior_month in 1..month {
        days += i64::from(match prior_month {
            2 if leap_year => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        });
    }
    days += i64::from(day - 1);
    days.checked_mul(86_400)
        .and_then(|seconds| seconds.checked_add(i64::from(hour) * 3_600))
        .and_then(|seconds| seconds.checked_add(i64::from(minute) * 60))
        .and_then(|seconds| seconds.checked_add(i64::from(second)))
        .and_then(|seconds| seconds.checked_sub(timezone_offset))
        .ok_or(())
}

const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn parse_byte_string(
    node: Node<'_, '_>,
    item_type: &str,
    allow_get_test_symbols: bool,
    allow_correlation_value: bool,
) -> Result<Vec<u8>, OasisCryptoFixtureError> {
    #[cfg(not(test))]
    let _ = allow_get_test_symbols;
    let value = value_attribute(node)?;
    if allow_correlation_value && item_type == "ByteString" && value == "$CORRELATION_VALUE" {
        return Ok(TEST_CORRELATION_VALUE.to_vec());
    }
    #[cfg(test)]
    if allow_get_test_symbols
        && item_type == "ByteString"
        && normalize_name(node.tag_name().name()) == "keymaterial"
        && matches!(value, "$KEY_MATERIAL" | "$KEY_MATERIAL_1")
    {
        // The pinned XML uses an unresolved source symbol here. Keep its
        // payload opaque while providing a zeroizing generic ByteString for
        // structural Any Object preservation tests.
        return Ok(vec![0xA5]);
    }
    reject_symbol(value)?;
    if value.len() % 2 != 0 {
        return Err(OasisCryptoFixtureError::InvalidValue);
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
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
