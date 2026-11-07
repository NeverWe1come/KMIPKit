//! KMIP wire names lookup for JSON and XML codecs.

#[path = "../../../kmipkit-client/src/wire_names_generated.rs"]
mod wire_names_generated;

use std::collections::BTreeMap;
use std::sync::OnceLock;
use wire_names_generated::WIRE_NAMES;

pub fn normalize(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut separated = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if matches!(c, '(' | ')')
            || (!c.is_alphanumeric()
                && c != '_'
                && chars.get(i + 1).is_some_and(|c| c.is_alphabetic())
                && chars.get(i + 2).is_some_and(|c| c.is_lowercase()))
        {
            separated.push(' ');
        } else if !c.is_alphanumeric() && c != '_' && !c.is_whitespace() {
            separated.push('_');
        } else {
            separated.push(c);
        }
    }
    let mut result = String::new();
    for (i, word) in separated.split_whitespace().enumerate() {
        let digits = if i == 0 {
            word.bytes().take_while(u8::is_ascii_digit).count()
        } else {
            0
        };
        let (prefix, rest) = word.split_at(digits);
        let mut chars = rest.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            result.extend(chars);
        }
        result.push_str(prefix);
    }
    result
}

#[derive(Default)]
struct Values {
    by_name: BTreeMap<String, u32>,
    by_raw: BTreeMap<u32, &'static str>,
}

#[derive(Default)]
struct Family {
    enumerations: Values,
    masks: Values,
}

#[derive(Default)]
struct Names {
    tags: BTreeMap<u32, &'static str>,
    tags_by_name: BTreeMap<String, u32>,
    enumerations: BTreeMap<u32, &'static str>,
    masks: BTreeMap<u32, &'static str>,
    families: BTreeMap<&'static str, Family>,
}

fn metadata() -> &'static Names {
    static NAMES: OnceLock<Names> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names = Names::default();
        for line in WIRE_NAMES.lines() {
            let mut fields = line.split('\t');
            let (Some(record), Some(first), Some(second)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            match record {
                "T" => {
                    if let Ok(tag) = u32::from_str_radix(first, 16) {
                        names.tags.insert(tag, second);
                        names.tags_by_name.insert(normalize(second), tag);
                    }
                }
                "F" | "M" => {
                    if let Ok(tag) = u32::from_str_radix(first, 16) {
                        if record == "F" {
                            names.enumerations.insert(tag, second);
                        } else {
                            names.masks.insert(tag, second);
                        }
                    }
                }
                "E" | "B" => {
                    if let (Ok(raw), Some(source)) =
                        (u32::from_str_radix(second, 16), fields.next())
                    {
                        let family = names.families.entry(first).or_default();
                        let values = if record == "E" {
                            &mut family.enumerations
                        } else {
                            &mut family.masks
                        };
                        values.by_name.insert(normalize(source), raw);
                        values.by_raw.insert(raw, source);
                    }
                }
                _ => (),
            }
        }
        names
    })
}

pub fn tag_name(tag: u32) -> Option<&'static str> {
    metadata().tags.get(&tag).copied()
}

pub fn tag_value(name: &str) -> Option<u32> {
    metadata().tags_by_name.get(name).copied()
}

#[derive(Clone, Copy)]
pub enum NameKind {
    Enumeration,
    Mask,
}

pub fn family(tag: u32, record: NameKind) -> Option<&'static str> {
    let names = metadata();
    match record {
        NameKind::Enumeration => names.enumerations.get(&tag).copied(),
        NameKind::Mask => names.masks.get(&tag).copied(),
    }
}

fn values(family: &str, record: NameKind) -> Option<&'static Values> {
    let family = metadata().families.get(family)?;
    match record {
        NameKind::Enumeration => Some(&family.enumerations),
        NameKind::Mask => Some(&family.masks),
    }
}

pub fn named_value(family: &str, name: &str, record: NameKind) -> Option<u32> {
    values(family, record)?.by_name.get(name).copied()
}

pub fn value_name(family: &str, value: u32, record: NameKind) -> Option<&'static str> {
    values(family, record)?.by_raw.get(&value).copied()
}
