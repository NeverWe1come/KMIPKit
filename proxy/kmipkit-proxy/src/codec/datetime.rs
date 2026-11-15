//! Lossless date-time formatting and parsing for KMIP JSON and XML codecs.

use std::fmt::Write;

pub fn days_from_civil(year: i128, month: i128, day: i128) -> i128 {
    let y = year - i128::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = month + if month > 2 { -3 } else { 9 };
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + (153 * m + 2) / 5 + day - 1 - 719_468
}

pub fn civil_from_days(days: i128) -> (i128, i128, i128) {
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

pub fn format_datetime(seconds: i64) -> String {
    let seconds = i128::from(seconds);
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let clock = seconds.rem_euclid(86_400);
    let h = clock / 3600;
    let m = clock / 60 % 60;
    let s = clock % 60;
    let mut text = String::with_capacity(32);
    if year <= 0 {
        let _ = write!(
            &mut text,
            "-{:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z",
            1 - year
        );
    } else {
        let _ = write!(
            &mut text,
            "{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z"
        );
    }
    text
}

pub fn parse_datetime(text: &str) -> Result<i64, String> {
    let (date, time) = text
        .split_once('T')
        .ok_or_else(|| "missing T".to_string())?;
    let (year_month, day) = date
        .rsplit_once('-')
        .ok_or_else(|| "missing date separator".to_string())?;
    let (year, month) = year_month
        .rsplit_once('-')
        .ok_or_else(|| "missing date separator".to_string())?;

    let year: i128 = year.parse().map_err(|e| format!("invalid year: {e}"))?;
    let astronomical = if year < 0 { year + 1 } else { year };
    let month: i128 = month.parse().map_err(|e| format!("invalid month: {e}"))?;
    let day: i128 = day.parse().map_err(|e| format!("invalid day: {e}"))?;

    let clock = time.strip_suffix('Z').unwrap_or(time);
    let mut parts = clock.split(':');
    let h: i128 = parts
        .next()
        .ok_or_else(|| "missing hour".to_string())?
        .parse()
        .map_err(|e| format!("invalid hour: {e}"))?;
    let m: i128 = parts
        .next()
        .ok_or_else(|| "missing minute".to_string())?
        .parse()
        .map_err(|e| format!("invalid minute: {e}"))?;
    let s: i128 = parts
        .next()
        .ok_or_else(|| "missing second".to_string())?
        .parse()
        .map_err(|e| format!("invalid second: {e}"))?;

    let days = days_from_civil(astronomical, month, day);
    let total_secs = days * 86_400 + h * 3600 + m * 60 + s;
    i64::try_from(total_secs).map_err(|e| format!("date out of range: {e}"))
}

pub fn hex_to_octets(hex: &str) -> Result<Vec<u8>, String> {
    let clean = hex.strip_prefix("0x").unwrap_or(hex);
    if clean.len() % 2 != 0 {
        return Err("odd number of hex digits".to_string());
    }
    let mut out = Vec::with_capacity(clean.len() / 2);
    for i in (0..clean.len()).step_by(2) {
        let byte =
            u8::from_str_radix(&clean[i..i + 2], 16).map_err(|e| format!("invalid hex: {e}"))?;
        out.push(byte);
    }
    Ok(out)
}

pub fn octets_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}
