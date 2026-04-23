use time::{OffsetDateTime, format_description::StaticFormatDescription, macros::format_description};

const FORMAT: StaticFormatDescription = format_description!("[year]-[month]-[day]_[hour]-[minute]-[second]");

pub fn pdf_name_for(facture_id: i64, now: &OffsetDateTime) -> String {
    let now_formatted = now.format(&FORMAT).expect("Formatting the datetime failed.");
    format!("facture_{}-{}.pdf", facture_id, now_formatted)
}
#[cfg(test)]
#[test]
fn test_pdf_name_for() {
    use time::{Date, Month, Time};

    let now = OffsetDateTime::new_utc(
        Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        Time::from_hms_nano(12, 59, 59, 500_000_000).unwrap(),
);
    assert_eq!(pdf_name_for(1, &now), "facture_1-2024-01-01_12-59-59.pdf")
}
