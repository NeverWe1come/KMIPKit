//! JSON scalar validation under OASIS KMIP Profiles v2.1 §5.5.1.6.
use super::{WireCodecError, WireCodecErrorKind, error};
use zeroize::Zeroizing;

pub(super) fn hex_u64(source: &str, bytes: usize) -> Result<u64, WireCodecError> {
    let text = source
        .strip_prefix("0x")
        .ok_or_else(|| error(WireCodecErrorKind::InvalidValue))?;
    if text.len() != bytes * 2 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(error(WireCodecErrorKind::InvalidValue));
    }
    u64::from_str_radix(text, 16).map_err(|_| error(WireCodecErrorKind::InvalidValue))
}
pub(super) fn hex_bytes(source: &str) -> Result<Zeroizing<Vec<u8>>, WireCodecError> {
    if !source.len().is_multiple_of(2) || !source.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(error(WireCodecErrorKind::InvalidValue));
    }
    let mut result = Zeroizing::new(Vec::new());
    result
        .try_reserve_exact(source.len() / 2)
        .map_err(|_| error(WireCodecErrorKind::AllocationFailed))?;
    for pair in source.as_bytes().as_chunks::<2>().0 {
        let high = char::from(pair[0])
            .to_digit(16)
            .ok_or_else(|| error(WireCodecErrorKind::InvalidValue))?;
        let low = char::from(pair[1])
            .to_digit(16)
            .ok_or_else(|| error(WireCodecErrorKind::InvalidValue))?;
        result.push(
            u8::try_from(high * 16 + low).map_err(|_| error(WireCodecErrorKind::InvalidValue))?,
        );
    }
    Ok(result)
}
pub(super) fn signed_number(source: &str) -> Result<i64, WireCodecError> {
    let n = source
        .parse::<i64>()
        .map_err(|_| error(WireCodecErrorKind::InvalidValue))?;
    if n.unsigned_abs() >= (1u64 << 52) {
        return Err(error(WireCodecErrorKind::InvalidValue));
    }
    Ok(n)
}

pub(super) fn date_time(source: &str) -> Result<i64, WireCodecError> {
    let invalid = || error(WireCodecErrorKind::InvalidValue);
    let (date, time) = source.split_once('T').ok_or_else(invalid)?;
    let negative = date.starts_with('-');
    let date = date.strip_prefix('-').unwrap_or(date);
    let mut parts = date.split('-');
    let year_text = parts.next().ok_or_else(invalid)?;
    if year_text.len() < 4 || !year_text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let mut year = year_text.parse::<i64>().map_err(|_| invalid())?;
    if year == 0 {
        return Err(invalid());
    } // XSD dateTime has no year zero.
    if negative {
        year = -year + 1;
    } // Convert XSD BCE to astronomical year numbering.
    let month = decimal(parts.next().ok_or_else(invalid)?, 2)?;
    let day = decimal(parts.next().ok_or_else(invalid)?, 2)?;
    if parts.next().is_some() || !(1..=12).contains(&month) {
        return Err(invalid());
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days).contains(&day) {
        return Err(invalid());
    }
    let (clock, offset) = if let Some(clock) = time.strip_suffix('Z') {
        (clock, 0i64)
    } else {
        let index = time
            .char_indices()
            .find_map(|(i, c)| matches!(c, '+' | '-').then_some(i))
            .ok_or_else(invalid)?;
        let (clock, zone) = time.split_at(index);
        let sign = if zone.starts_with('-') { -1 } else { 1 };
        let (h, m) = zone[1..].split_once(':').ok_or_else(invalid)?;
        let h = decimal(h, 2)?;
        let m = decimal(m, 2)?;
        if h > 14 || m > 59 || (h == 14 && m != 0) {
            return Err(invalid());
        }
        (clock, sign * (h * 3600 + m * 60))
    };
    let mut parts = clock.split(':');
    let hour = decimal(parts.next().ok_or_else(invalid)?, 2)?;
    let minute = decimal(parts.next().ok_or_else(invalid)?, 2)?;
    let second = parts.next().ok_or_else(invalid)?;
    if parts.next().is_some() {
        return Err(invalid());
    }
    let fraction_nonzero = second
        .split_once('.')
        .is_some_and(|(_, fraction)| fraction.bytes().any(|b| b != b'0'));
    let second = if let Some((whole, fraction)) = second.split_once('.') {
        if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        whole
    } else {
        second
    };
    let second = decimal(second, 2)?;
    if hour > 24
        || minute > 59
        || second > 59
        || (hour == 24 && (minute != 0 || second != 0 || fraction_nonzero))
    {
        return Err(invalid());
    }
    // Gregorian civil-date conversion; checked arithmetic rejects unrepresentable epochs.
    let year = year
        .checked_sub(i64::from(month <= 2))
        .ok_or_else(invalid)?;
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let shifted = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era.checked_mul(146_097)
        .and_then(|n| n.checked_add(doe))
        .and_then(|n| n.checked_sub(719_468))
        .and_then(|n| n.checked_mul(86400))
        .and_then(|n| n.checked_add(hour * 3600 + minute * 60 + second))
        .and_then(|n| n.checked_sub(offset))
        .ok_or_else(invalid)
}
fn decimal(source: &str, width: usize) -> Result<i64, WireCodecError> {
    if source.len() != width || !source.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(WireCodecErrorKind::InvalidValue));
    }
    source
        .parse()
        .map_err(|_| error(WireCodecErrorKind::InvalidValue))
}
