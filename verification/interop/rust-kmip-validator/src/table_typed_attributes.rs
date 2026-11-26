use crate::{decode_hex, generic_wire};
use kmipkit::protocol::attribute::typed;
use kmipkit::protocol::{
    Attribute, AttributeDefinition, AttributeValueError, UniqueIdentifierValue,
    UniqueIdentifierView,
};
use kmipkit::ttlv::{Item, ModelError, Structure, StructureView};
use serde_json::json;
use std::error::Error;
use std::io::{BufRead, BufWriter, Write};

type Verdicts = (&'static str, &'static str);

pub(crate) fn inspect() -> Result<(), Box<dyn Error>> {
    let mut output = BufWriter::new(std::io::stdout().lock());
    writeln!(
        output,
        "{}",
        json!({
            "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"),
            "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256"),
        })
    )?;
    for line in std::io::stdin().lock().lines() {
        let input = line?;
        let mut parts = input.split_whitespace();
        let tag = parts.next().ok_or("missing attribute tag")?.parse()?;
        let input =
            kmipkit::ttlv::codec::decode(&decode_hex(parts.next().ok_or("missing input item")?)?)?;
        let expected = decode_hex(parts.next().ok_or("missing expected item")?)?;
        let (construct, read) = attribute_verdicts(tag, input, &expected)?;
        writeln!(output, "{}", json!({"construct": construct, "read": read}))?;
    }
    output.flush()?;
    Ok(())
}

fn attribute_verdicts(tag: u32, input: Item, expected: &[u8]) -> Result<Verdicts, Box<dyn Error>> {
    macro_rules! check_markers {
        ($($marker:ident => $owned:ident),+ $(,)?) => {
            match tag {
                $(<typed::$marker as AttributeDefinition>::TAG => {
                    check::<typed::$marker>(input, expected, $owned)
                },)+
                _ => Err("unknown typed attribute tag".into()),
            }
        };
    }

    check_markers!(
        ActivationDate => copy_value,
        AlternativeName => structure_value,
        AlwaysSensitive => copy_value,
        ApplicationSpecificInformation => structure_value,
        ArchiveDate => copy_value,
        CertificateLength => copy_value,
        CertificateType => copy_value,
        Comment => text_value,
        CompromiseDate => copy_value,
        CompromiseOccurrenceDate => copy_value,
        ContactInformation => text_value,
        CryptographicAlgorithm => copy_value,
        CryptographicDomainParameters => structure_value,
        CryptographicLength => copy_value,
        CryptographicParameters => structure_value,
        CryptographicUsageMask => copy_value,
        DeactivationDate => copy_value,
        Description => text_value,
        DestroyDate => copy_value,
        Digest => structure_value,
        DigitalSignatureAlgorithm => copy_value,
        Extractable => copy_value,
        Fresh => copy_value,
        InitialDate => copy_value,
        KeyFormatType => copy_value,
        KeyValueLocation => structure_value,
        KeyValuePresent => copy_value,
        LastChangeDate => copy_value,
        LeaseTime => copy_value,
        Link => structure_value,
        Name => structure_value,
        NeverExtractable => copy_value,
        NistKeyType => copy_value,
        ObjectGroup => text_value,
        ObjectType => copy_value,
        OpaqueDataType => copy_value,
        OriginalCreationDate => copy_value,
        Pkcs12FriendlyName => text_value,
        ProcessStartDate => copy_value,
        ProtectStopDate => copy_value,
        ProtectionLevel => copy_value,
        ProtectionPeriod => copy_value,
        QuantumSafe => copy_value,
        RandomNumberGenerator => structure_value,
        RevocationReason => structure_value,
        RotateAutomatic => copy_value,
        RotateDate => copy_value,
        RotateGeneration => copy_value,
        RotateInterval => copy_value,
        RotateLatest => copy_value,
        RotateName => structure_value,
        RotateOffset => copy_value,
        Sensitive => copy_value,
        ShortUniqueIdentifier => bytes_value,
        State => copy_value,
        UniqueIdentifier => identifier_value,
        UsageLimits => structure_value,
        VendorAttribute => structure_value,
        X509CertificateIdentifier => structure_value,
        X509CertificateIssuer => structure_value,
        X509CertificateSubject => structure_value,
    )
}

fn check<D: AttributeDefinition>(
    input: Item,
    expected: &[u8],
    owned: impl for<'a> FnOnce(&D::ValueView<'a>) -> Result<D::Value, ModelError>,
) -> Result<Verdicts, Box<dyn Error>> {
    let value = input.with_value(|value| D::value_view(value).map(|view| owned(&view)));
    let construct = match value {
        Ok(value) => verdict(Attribute::<D>::new(value?)?.as_item(), expected),
        // This constructor takes D::Value. The invalid Item is tested through
        // from_item below, where a mismatched TTLV type can be supplied.
        Err(AttributeValueError::ValueTypeMismatch) => "not-applicable",
        Err(error) => return Err(error.into()),
    };
    let read = match Attribute::<D>::from_item(input) {
        Ok(attribute) => verdict(attribute.as_item(), expected),
        Err(AttributeValueError::TagMismatch | AttributeValueError::ValueTypeMismatch) => {
            "rejected"
        }
        Err(error) => return Err(error.into()),
    };
    Ok((construct, read))
}

fn verdict(item: &Item, expected: &[u8]) -> &'static str {
    let mut encoded = Vec::new();
    if generic_wire::encode(item, &mut encoded).is_ok() && encoded == expected {
        "passed"
    } else {
        "failed"
    }
}

fn structure_value(value: &StructureView<'_>) -> Result<Structure, ModelError> {
    value.try_clone()
}

#[allow(clippy::unnecessary_wraps)]
fn copy_value<T: Copy>(value: &T) -> Result<T, ModelError> {
    Ok(*value)
}

#[allow(clippy::unnecessary_wraps)]
fn text_value(value: &&str) -> Result<String, ModelError> {
    Ok((*value).to_owned())
}

#[allow(clippy::unnecessary_wraps)]
fn bytes_value(value: &&[u8]) -> Result<Vec<u8>, ModelError> {
    Ok((*value).to_owned())
}

#[allow(clippy::unnecessary_wraps)]
fn identifier_value(value: &UniqueIdentifierView<'_>) -> Result<UniqueIdentifierValue, ModelError> {
    Ok(match *value {
        UniqueIdentifierView::TextString(value) => {
            UniqueIdentifierValue::TextString(value.to_owned())
        }
        UniqueIdentifierView::Enumeration(value) => UniqueIdentifierValue::Enumeration(value),
        UniqueIdentifierView::Integer(value) => UniqueIdentifierValue::Integer(value),
        UniqueIdentifierView::Identifier(value) => {
            UniqueIdentifierValue::Identifier(value.to_owned())
        }
    })
}
