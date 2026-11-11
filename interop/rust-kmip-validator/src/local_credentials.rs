//! Local credential fixtures and public calls: OASIS KMIP 2.1 §§9.4, 9.11, 9.14.
use crate::{
    driver::{item, structure},
    models::Result,
};
use kmipkit::protocol::{
    AttestationCredential, Authentication, Credential, CredentialType, CredentialValue,
    DeviceCredential, HashedPasswordCredential, Nonce, UsernameAndPasswordCredential,
};
use kmipkit::ttlv::{Item, Value, ValueView};
use serde_json::Value as Json;

fn text(tag: u32) -> Result<Item> {
    item(tag, Value::text_string("KMIPKit".to_owned()))
}
fn bytes(tag: u32) -> Result<Item> {
    item(tag, Value::byte_string(vec![0x4b]))
}
fn node(tag: u32, children: Vec<Item>) -> Result<Item> {
    item(tag, Value::structure(structure(tag, children)?))
}
fn copy(item: &Item) -> Result<Item> {
    item.try_clone()
        .map_err(|_| "credential fixture copy failed")
}
fn body(item: &Item) -> Result<kmipkit::ttlv::Structure> {
    item.as_structure()
        .ok_or("credential fixture type mismatch")?
        .try_clone()
        .map_err(|_| "credential fixture copy failed")
}

fn kind(target_id: &str) -> u32 {
    if target_id.contains("ATTESTATION") {
        3
    } else if target_id.contains("HASHED-PASSWORD") {
        5
    } else if target_id.contains("ONE-TIME-PASSWORD") {
        4
    } else if target_id.contains("DEVICE") {
        2
    } else if target_id.contains("TICKET") {
        6
    } else {
        1
    }
}

fn replace(tree: Item, target: Option<&Item>) -> Result<Item> {
    if let Some(target) = target
        && tree.tag().raw() == target.tag().raw()
    {
        return copy(target);
    }
    if let Some(children) = tree.as_structure() {
        let children = children
            .children()
            .iter()
            .map(|child| replace(copy(child)?, target))
            .collect::<Result<Vec<_>>>()?;
        return node(tree.tag().raw(), children);
    }
    Ok(tree)
}

fn nonce(target: Option<&Item>) -> Result<Item> {
    replace(
        node(0x0042_00c8, vec![bytes(0x0042_00c9)?, bytes(0x0042_00ca)?])?,
        target,
    )
}

fn value(kind: u32, target: Option<&Item>) -> Result<Item> {
    let fields = match kind {
        2 => vec![
            text(0x0042_00b0)?,
            text(0x0042_00a1)?,
            text(0x0042_00a2)?,
            text(0x0042_00ab)?,
            text(0x0042_00a9)?,
            text(0x0042_00aa)?,
        ],
        3 => vec![
            nonce(target)?,
            item(0x0042_00c7, Value::enumeration(1))?,
            bytes(0x0042_00cb)?,
            bytes(0x0042_00cc)?,
        ],
        4 => vec![text(0x0042_0099)?, text(0x0042_00a1)?, text(0x0042_0156)?],
        5 => vec![
            text(0x0042_0099)?,
            item(0x0042_0092, Value::date_time_extended(1))?,
            item(0x0042_0038, Value::enumeration(6))?,
            bytes(0x0042_0157)?,
        ],
        6 => vec![node(
            0x0042_0149,
            vec![
                item(0x0042_014a, Value::enumeration(1))?,
                bytes(0x0042_014b)?,
            ],
        )?],
        _ => vec![text(0x0042_0099)?, text(0x0042_00a1)?],
    };
    replace(node(0x0042_0025, fields)?, target)
}

fn credential(kind: u32, target: Option<&Item>) -> Result<Item> {
    replace(
        node(
            0x0042_0023,
            vec![
                item(0x0042_0024, Value::enumeration(kind))?,
                value(kind, target)?,
            ],
        )?,
        target,
    )
}

pub(crate) fn fixture(target: &Json) -> Option<Result<Item>> {
    let target_id = target["target_id"].as_str()?;
    let name = target["name"].as_str()?;
    if target["kind"] == "credential" {
        return Some(if target_id == "KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL" {
            credential(1, None)
        } else {
            value(kind(target_id), None)
        });
    }
    match name {
        "Nonce" => Some(nonce(None)),
        "Credential" => Some(credential(1, None)),
        "Credential Value" => Some(value(1, None)),
        "Authentication" => {
            Some(credential(1, None).and_then(|credential| node(0x0042_000c, vec![credential])))
        }
        _ => None,
    }
}

macro_rules! variant {
    ($symbol:expr, $owner:ty, $kind:expr, $sample:expr) => {{
        let input = value($kind, Some($sample)).ok()?;
        let input = body(&input).ok()?;
        if $symbol.ends_with("::try_from_ttlv") {
            Some(
                <$owner>::try_from_ttlv(input)
                    .map_err(|_| "credential constructor rejected")
                    .and_then(|model| {
                        node(
                            0x0042_0025,
                            model
                                .into_ttlv()
                                .view()
                                .children()
                                .iter()
                                .map(copy)
                                .collect::<Result<Vec<_>>>()?,
                        )
                    }),
            )
        } else {
            let model = <$owner>::try_from_ttlv(input).ok()?;
            Some(item(0x0042_0025, Value::structure(model.into_ttlv())))
        }
    }};
}

#[allow(clippy::too_many_lines)] // Each exact public route remains visible next to its prerequisite.
pub(crate) fn invoke(symbol: &str, sample: &Item, target_id: &str) -> Option<Result<Item>> {
    let selected = symbol.rsplit("::").next()?;
    if !["try_from_ttlv", "into_ttlv", "new"].contains(&selected) {
        return None;
    }
    if symbol.starts_with("kmipkit_protocol::UsernameAndPasswordCredential::") {
        return variant!(symbol, UsernameAndPasswordCredential, 1, sample);
    }
    if symbol.starts_with("kmipkit_protocol::DeviceCredential::") {
        return variant!(symbol, DeviceCredential, 2, sample);
    }
    if symbol.starts_with("kmipkit_protocol::AttestationCredential::") {
        return variant!(symbol, AttestationCredential, 3, sample);
    }
    if symbol.starts_with("kmipkit_protocol::HashedPasswordCredential::") {
        return variant!(symbol, HashedPasswordCredential, 5, sample);
    }
    if symbol.starts_with("kmipkit_protocol::Nonce::") {
        let input = body(&nonce(Some(sample)).ok()?).ok()?;
        if symbol.ends_with("::try_from_ttlv") {
            return Some(
                Nonce::try_from_ttlv(input)
                    .map_err(|_| "nonce constructor rejected")
                    .and_then(|model| item(0x0042_00c8, Value::structure(model.into_ttlv()))),
            );
        }
        let model = Nonce::try_from_ttlv(input).ok()?;
        return Some(item(0x0042_00c8, Value::structure(model.into_ttlv())));
    }
    if symbol.starts_with("kmipkit_protocol::CredentialValue::") {
        let kind = kind(target_id);
        let input = body(&value(kind, Some(sample)).ok()?).ok()?;
        if symbol.ends_with("::try_from_ttlv") {
            return Some(
                CredentialValue::try_from_ttlv(CredentialType::from_raw(kind), input)
                    .map_err(|_| "credential constructor rejected")
                    .and_then(|model| item(0x0042_0025, Value::structure(model.into_ttlv()))),
            );
        }
        let model = CredentialValue::try_from_ttlv(CredentialType::from_raw(kind), input).ok()?;
        return Some(item(0x0042_0025, Value::structure(model.into_ttlv())));
    }
    if symbol.starts_with("kmipkit_protocol::Credential::") {
        let mut kind = kind(target_id);
        if sample.tag().raw() == 0x0042_0024 {
            kind = sample.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })?;
        }
        let input = body(&credential(kind, Some(sample)).ok()?).ok()?;
        if symbol.ends_with("::try_from_ttlv") {
            return Some(
                Credential::try_from_ttlv(input)
                    .map_err(|_| "credential constructor rejected")
                    .and_then(|model| item(0x0042_0023, Value::structure(model.into_ttlv()))),
            );
        }
        let model = Credential::try_from_ttlv(input).ok()?;
        return Some(item(0x0042_0023, Value::structure(model.into_ttlv())));
    }
    if symbol.starts_with("kmipkit_protocol::Authentication::") {
        let input = if sample.tag().raw() == 0x0042_000c {
            body(sample).ok()?
        } else {
            body(
                &node(
                    0x0042_000c,
                    vec![credential(kind(target_id), Some(sample)).ok()?],
                )
                .ok()?,
            )
            .ok()?
        };
        if symbol.ends_with("::try_from_ttlv") {
            return Some(
                Authentication::try_from_ttlv(input)
                    .map_err(|_| "authentication constructor rejected")
                    .and_then(|model| item(0x0042_000c, Value::structure(model.into_ttlv()))),
            );
        }
        if symbol.ends_with("::new") {
            let credentials = input
                .view()
                .children()
                .iter()
                .map(|child| {
                    Credential::try_from_ttlv(body(child)?)
                        .map_err(|_| "credential fixture rejected")
                })
                .collect::<Result<Vec<_>>>()
                .ok()?;
            return Some(
                Authentication::new(credentials)
                    .map_err(|_| "authentication constructor rejected")
                    .and_then(|model| item(0x0042_000c, Value::structure(model.into_ttlv()))),
            );
        }
        let model = Authentication::try_from_ttlv(input).ok()?;
        return Some(item(0x0042_000c, Value::structure(model.into_ttlv())));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_wire::target_hash;

    #[test]
    fn credential_value_route_builds_the_branch_containing_its_target_tag() {
        let sample = item(0x0042_0038, Value::enumeration(6)).unwrap();
        let result = invoke(
            "kmipkit_protocol::CredentialValue::into_ttlv",
            &sample,
            "KMIPKIT-ELEM-TAG-420038",
        )
        .unwrap()
        .unwrap();

        assert!(target_hash(&result, &sample).is_ok());
    }
}
