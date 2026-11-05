//! Lossless XML scalar values; Profiles v2.1 §§5.4.1.6.3–5.4.1.6.13.
use super::characters::xml_space;
use super::names::{enum_value, family, hex_number, normalize, rows};
use super::{ALLOCATION, INVALID_VALUE, Result};
use kmipkit_ttlv::Value;
use zeroize::Zeroizing;

fn integer(raw: u32, value: &str) -> Result<i32> {
    if let Ok(number) = value.parse::<i32>() {
        return Ok(number);
    }
    let mut bits = 0_u32;
    let mut found = false;
    let mask_family = family(raw, "M");
    for component in value.split_ascii_whitespace() {
        if found && mask_family.is_none() {
            return Err(INVALID_VALUE);
        }
        let part = if component.starts_with("0x") {
            u32::try_from(hex_number(component, 8)?).map_err(|_| INVALID_VALUE)?
        } else {
            let family = mask_family.ok_or(INVALID_VALUE)?;
            rows("B")
                .find(|r| r[1] == family && normalize(r[3]) == component)
                .and_then(|r| u32::from_str_radix(r[2], 16).ok())
                .ok_or(INVALID_VALUE)?
        };
        found = true;
        bits |= part;
    }
    if !found {
        return Err(INVALID_VALUE);
    }
    Ok(i32::from_be_bytes(bits.to_be_bytes()))
}

fn signed_long(value: &str) -> Result<i64> {
    if value.starts_with("0x") {
        return Ok(i64::from_be_bytes(hex_number(value, 16)?.to_be_bytes()));
    }
    value.parse().map_err(|_| INVALID_VALUE)
}

fn octets(text: &str, big: bool) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2)
        || (big && (text.is_empty() || !text.len().is_multiple_of(16)))
        || !text.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(INVALID_VALUE);
    }
    let mut bytes = Zeroizing::new(Vec::new());
    bytes
        .try_reserve_exact(text.len() / 2)
        .map_err(|_| ALLOCATION)?;
    for pair in text.as_bytes().as_chunks::<2>().0 {
        let digit = |c: u8| {
            if c.is_ascii_digit() {
                c - b'0'
            } else {
                c.to_ascii_uppercase() - b'A' + 10
            }
        };
        bytes.push(digit(pair[0]) * 16 + digit(pair[1]));
    }
    Ok(std::mem::take(&mut *bytes))
}

pub(super) fn scalar(raw: u32, ty: &str, value: &str) -> Result<Value> {
    let collapsed = value.trim_matches(xml_space);
    Ok(match ty {
        "Integer" => Value::integer(integer(raw, collapsed)?),
        "LongInteger" => Value::long_integer(signed_long(collapsed)?),
        "BigInteger" => Value::big_integer(octets(collapsed, true)?),
        "Enumeration" => Value::enumeration(enum_value(raw, value)?),
        "Boolean" => Value::boolean(match collapsed {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => return Err(INVALID_VALUE),
        }),
        "TextString" => {
            let mut text = Zeroizing::new(String::new());
            text.try_reserve_exact(value.len())
                .map_err(|_| ALLOCATION)?;
            text.push_str(value);
            Value::text_string(std::mem::take(&mut *text))
        }
        "ByteString" => Value::byte_string(octets(collapsed, false)?),
        "DateTime" => Value::date_time(parse_datetime(collapsed)?),
        "Interval" => Value::interval(collapsed.parse().map_err(|_| INVALID_VALUE)?),
        "DateTimeExtended" => {
            Value::date_time_extended(collapsed.parse().map_err(|_| INVALID_VALUE)?)
        }
        _ => return Err(INVALID_VALUE),
    })
}

// Gregorian conversion uses Euclidean 400-year eras. i128 arithmetic covers
// every i64 epoch second without narrowing its XML year (§5.4.1.6.11).
fn days_from_civil(year: i128, month: i128, day: i128) -> i128 {
    let y = year - i128::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = month + if month > 2 { -3 } else { 9 };
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + (153 * m + 2) / 5 + day - 1 - 719_468
}

fn civil_from_days(days: i128) -> (i128, i128, i128) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + i128::from(m <= 2), m, d)
}

fn two_digits(value: &str) -> Result<i128> {
    if value.len() != 2 || !value.bytes().all(|c| c.is_ascii_digit()) {
        return Err(INVALID_VALUE);
    }
    value.parse().map_err(|_| INVALID_VALUE)
}

fn parse_datetime(text: &str) -> Result<i64> {
    let (date, time) = text.split_once('T').ok_or(INVALID_VALUE)?;
    let (year_month, day) = date.rsplit_once('-').ok_or(INVALID_VALUE)?;
    let (year, month) = year_month.rsplit_once('-').ok_or(INVALID_VALUE)?;
    let unsigned_year = year.strip_prefix('-').unwrap_or(year);
    if unsigned_year.len() < 4
        || !unsigned_year.bytes().all(|c| c.is_ascii_digit())
        || unsigned_year.len() > 18
        || (unsigned_year.len() > 4 && unsigned_year.starts_with('0'))
    {
        return Err(INVALID_VALUE);
    }
    let year: i128 = year.parse().map_err(|_| INVALID_VALUE)?;
    if year == 0 {
        return Err(INVALID_VALUE);
    }
    let astronomical = if year < 0 { year + 1 } else { year };
    let month = two_digits(month)?;
    let day = two_digits(day)?;
    let leap = astronomical.rem_euclid(4) == 0
        && (astronomical.rem_euclid(100) != 0 || astronomical.rem_euclid(400) == 0);
    let max = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return Err(INVALID_VALUE),
    };
    if day < 1 || day > max {
        return Err(INVALID_VALUE);
    }
    let (clock, zone) = if let Some(clock) = time.strip_suffix('Z') {
        (clock, 0)
    } else {
        let index = time.find(['+', '-']).ok_or(INVALID_VALUE)?;
        let (h, m) = time[index + 1..].split_once(':').ok_or(INVALID_VALUE)?;
        let h = two_digits(h)?;
        let m = two_digits(m)?;
        if h > 14 || m > 59 || (h == 14 && m != 0) {
            return Err(INVALID_VALUE);
        }
        (
            &time[..index],
            (h * 3600 + m * 60)
                * if time.as_bytes()[index] == b'-' {
                    -1
                } else {
                    1
                },
        )
    };
    let (h, rest) = clock.split_once(':').ok_or(INVALID_VALUE)?;
    let (m, second) = rest.split_once(':').ok_or(INVALID_VALUE)?;
    let second = if let Some((s, fraction)) = second.split_once('.') {
        if fraction.is_empty() || !fraction.bytes().all(|c| c == b'0') {
            return Err(INVALID_VALUE);
        }
        s
    } else {
        second
    };
    let h = two_digits(h)?;
    let m = two_digits(m)?;
    let s = two_digits(second)?;
    if h > 24 || m > 59 || s > 59 || (h == 24 && (m != 0 || s != 0)) {
        return Err(INVALID_VALUE);
    }
    let seconds = days_from_civil(astronomical, month, day) * 86_400 + h * 3600 + m * 60 + s - zone;
    i64::try_from(seconds).map_err(|_| INVALID_VALUE)
}

pub(super) fn datetime(seconds: i64) -> Zeroizing<String> {
    let seconds = i128::from(seconds);
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let clock = seconds.rem_euclid(86_400);
    let h = clock / 3600;
    let m = clock / 60 % 60;
    let s = clock % 60;
    if year <= 0 {
        Zeroizing::new(format!(
            "-{:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z",
            1 - year
        ))
    } else {
        Zeroizing::new(format!(
            "{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z"
        ))
    }
}
