use super::*;

fn secs(text: &str) -> u64 {
    match parse(text) {
        Ok(length) => length.as_secs(),
        Err(e) => panic!("{text} did not parse: {e}"),
    }
}

#[test]
fn parse_minutes() {
    assert_eq!(secs("10m"), 600);
}

#[test]
fn parse_seconds() {
    assert_eq!(secs("90s"), 90);
}

#[test]
fn parse_hours_and_minutes() {
    assert_eq!(secs("1h30m"), 5400);
}

#[test]
fn parse_bare_number_is_minutes() {
    assert_eq!(secs("25"), 1500);
}

#[test]
fn parse_ignores_case_and_spaces() {
    assert_eq!(secs(" 2M "), 120);
}

#[test]
fn parse_colon_form() {
    assert_eq!(secs("1:30"), 90);
}

#[test]
fn parse_colon_form_with_hours() {
    assert_eq!(secs("1:05:00"), 3900);
}

#[test]
fn parse_rejects_bad_input() {
    let bad = ["", "abc", "10x", "5m3", "1:3", "1:75", "1:2:3:4", "m"];
    let accepted: Vec<&str> = bad.into_iter().filter(|t| parse(t).is_ok()).collect();
    assert_eq!(accepted, Vec::<&str>::new());
}

#[test]
fn parse_rejects_zero() {
    assert_eq!(
        parse("0m").map_err(|e| e.to_string()),
        Err("a timer needs at least one second, try 25m".to_string())
    );
}

#[test]
fn parse_rejects_overflow() {
    assert!(parse("99999999999999999999h").is_err());
}

#[test]
fn format_under_an_hour() {
    assert_eq!(clock(1500), "25:00");
}

#[test]
fn format_pads_seconds() {
    assert_eq!(clock(65), "01:05");
}

#[test]
fn format_over_an_hour() {
    assert_eq!(clock(5405), "1:30:05");
}

#[test]
fn format_short_lengths() {
    let got: Vec<String> = [1500, 5400, 90, 45, 0].into_iter().map(short).collect();
    assert_eq!(got, ["25m", "1h 30m", "1m 30s", "45s", "0s"]);
}
