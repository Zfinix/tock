//! Reading timer lengths like `25m`, `1h30m` or `1:30`, and writing them back.

use std::time::Duration;

use anyhow::{Result, bail};

/// Reads a timer length: units (`10m`, `90s`, `1h30m`), a clock (`1:30` is a
/// minute and a half, `1:00:00` an hour), or a bare number of minutes.
pub fn parse(text: &str) -> Result<Duration> {
    let text = text.trim().to_ascii_lowercase();
    let secs = match text.contains(':') {
        true => clock_secs(&text),
        false => unit_secs(&text),
    };
    let Some(secs) = secs else {
        bail!("could not read \"{text}\" as a length, try 25m, 90s, 1h30m or 1:30");
    };
    if secs == 0 {
        bail!("a timer needs at least one second, try 25m");
    }
    Ok(Duration::from_secs(secs))
}

fn clock_secs(text: &str) -> Option<u64> {
    let parts: Vec<&str> = text.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }
    let mut total: u64 = 0;
    for (i, part) in parts.iter().enumerate() {
        let value: u64 = part.parse().ok()?;
        if i > 0 && (part.len() != 2 || value >= 60) {
            return None;
        }
        total = total.checked_mul(60)?.checked_add(value)?;
    }
    Some(total)
}

fn unit_secs(text: &str) -> Option<u64> {
    if let Ok(minutes) = text.parse::<u64>() {
        return minutes.checked_mul(60);
    }
    let mut total: u64 = 0;
    let mut digits = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
            continue;
        }
        let scale = match c {
            'h' => 3600,
            'm' => 60,
            's' => 1,
            _ => return None,
        };
        let value: u64 = digits.parse().ok()?;
        total = total.checked_add(value.checked_mul(scale)?)?;
        digits.clear();
    }
    match digits.is_empty() {
        true => Some(total),
        false => None,
    }
}

/// A countdown readout: `mm:ss`, or `h:mm:ss` from an hour up.
pub fn clock(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    match h > 0 {
        true => format!("{h}:{m:02}:{s:02}"),
        false => format!("{m:02}:{s:02}"),
    }
}

/// A length for a summary line: `25m`, `1h 30m`, `1m 30s`, `45s`.
pub fn short(secs: u64) -> String {
    let parts: Vec<String> = [
        (secs / 3600, "h"),
        (secs % 3600 / 60, "m"),
        (secs % 60, "s"),
    ]
    .into_iter()
    .filter(|(value, _)| *value > 0)
    .map(|(value, unit)| format!("{value}{unit}"))
    .collect();
    match parts.is_empty() {
        true => "0s".to_string(),
        false => parts.join(" "),
    }
}

#[cfg(test)]
#[path = "duration_test.rs"]
mod tests;
