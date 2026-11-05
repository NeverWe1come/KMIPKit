//! Profile-normalized names from the private checked-in metadata only.
use crate::wire_names_generated::WIRE_NAMES;

pub(super) fn normalize(source: &str) -> String {
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

pub(super) fn tag_name(tag: u32) -> Option<&'static str> {
    WIRE_NAMES.lines().find_map(|line| {
        let mut f = line.split('\t');
        if f.next() != Some("T") {
            return None;
        }
        let raw = u32::from_str_radix(f.next()?, 16).ok()?;
        (raw == tag).then(|| f.next()).flatten()
    })
}
pub(super) fn tag_value(name: &str) -> Option<u32> {
    WIRE_NAMES.lines().find_map(|line| {
        let mut f = line.split('\t');
        if f.next() != Some("T") {
            return None;
        }
        let raw = f.next()?;
        (normalize(f.next()?) == name)
            .then(|| u32::from_str_radix(raw, 16).ok())
            .flatten()
    })
}
pub(super) fn family(tag: u32, record: char) -> Option<&'static str> {
    WIRE_NAMES.lines().find_map(|line| {
        let mut f = line.split('\t');
        if !f.next()?.starts_with(record) {
            return None;
        }
        let raw = u32::from_str_radix(f.next()?, 16).ok()?;
        (raw == tag).then(|| f.next()).flatten()
    })
}
pub(super) fn named_value(family: &str, name: &str, record: char) -> Option<u32> {
    WIRE_NAMES.lines().find_map(|line| {
        let mut f = line.split('\t');
        if !f.next()?.starts_with(record) || f.next()? != family {
            return None;
        }
        let raw = f.next()?;
        (normalize(f.next()?) == name)
            .then(|| u32::from_str_radix(raw, 16).ok())
            .flatten()
    })
}
pub(super) fn value_name(family: &str, value: u32, record: char) -> Option<&'static str> {
    WIRE_NAMES.lines().find_map(|line| {
        let mut f = line.split('\t');
        if !f.next()?.starts_with(record) || f.next()? != family {
            return None;
        }
        let raw = u32::from_str_radix(f.next()?, 16).ok()?;
        (raw == value).then(|| f.next()).flatten()
    })
}
