/// Formats an i64 cent value as a decimal dollar string.
/// e.g. 1150 -> "11.50", -500 -> "-5.00", 0 -> "0.00"
pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{}{}.{:02}", sign, abs / 100, abs % 100)
}
