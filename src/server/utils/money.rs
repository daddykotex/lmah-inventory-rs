/// Formats an i64 cent value as a decimal dollar string.
/// e.g. 1150 -> "11.50", -500 -> "-5.00", 0 -> "0.00"
pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{}{}.{:02}", sign, abs / 100, abs % 100)
}

/// Parses a decimal dollar string into i64 cents.
/// Parses the input as f64, rounds to nearest cent, then converts.
/// e.g. "11.50" -> Ok(1150), "11" -> Ok(1100), "-5.00" -> Ok(-500)
pub fn parse_money(s: &str) -> Result<i64, String> {
    let value: f64 = s
        .trim()
        .parse()
        .map_err(|e| format!("invalid amount: {}", e))?;
    Ok((value * 100.0).round() as i64)
}
