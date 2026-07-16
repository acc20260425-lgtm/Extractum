use time::macros::format_description;
use time::{Date, PrimitiveDateTime, Time};

pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn now_rfc3339_utc() -> String {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    OffsetDateTime::from_unix_timestamp(now_secs())
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .format(&Rfc3339)
        .expect("format current UTC timestamp")
}

pub fn ymd_to_unix_midnight(value: &str) -> Option<i64> {
    let value = value.trim();

    let date = if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        Date::parse(value, format_description!("[year][month][day]")).ok()
    } else {
        Date::parse(value, format_description!("[year]-[month]-[day]")).ok()
    }?;

    Some(
        PrimitiveDateTime::new(date, Time::MIDNIGHT)
            .assume_utc()
            .unix_timestamp(),
    )
}

#[cfg(test)]
mod tests {
    use super::{now_rfc3339_utc, now_secs, ymd_to_unix_midnight};

    #[test]
    fn ymd_to_unix_midnight_parses_iso_dates() {
        assert_eq!(ymd_to_unix_midnight("1970-01-01"), Some(0));
        assert_eq!(ymd_to_unix_midnight("2024-01-02"), Some(1_704_153_600));
    }

    #[test]
    fn ymd_to_unix_midnight_parses_compact_youtube_dates() {
        assert_eq!(ymd_to_unix_midnight("20240102"), Some(1_704_153_600));
    }

    #[test]
    fn ymd_to_unix_midnight_rejects_malformed_dates() {
        assert_eq!(ymd_to_unix_midnight("2024-00-02"), None);
        assert_eq!(ymd_to_unix_midnight("2024-01-00"), None);
        assert_eq!(ymd_to_unix_midnight("2024-13-02"), None);
        assert_eq!(ymd_to_unix_midnight("2024-01-02-extra"), None);
        assert_eq!(ymd_to_unix_midnight("not-a-date"), None);
    }

    #[test]
    fn ymd_to_unix_midnight_rejects_nonexistent_calendar_dates() {
        assert_eq!(ymd_to_unix_midnight("2024-02-30"), None);
        assert_eq!(ymd_to_unix_midnight("2023-02-29"), None);
        assert_eq!(ymd_to_unix_midnight("2024-02-29"), Some(1_709_164_800));
    }

    #[test]
    fn ymd_to_unix_midnight_rejects_non_canonical_iso_dates() {
        assert_eq!(ymd_to_unix_midnight("2024-1-02"), None);
        assert_eq!(ymd_to_unix_midnight("2024-01-2"), None);
    }

    #[test]
    fn now_secs_returns_unix_timestamp_seconds() {
        assert!(now_secs() >= 1_700_000_000);
    }

    #[test]
    fn now_rfc3339_utc_returns_current_utc_timestamp() {
        use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

        let before = OffsetDateTime::now_utc() - Duration::seconds(5);
        let value = now_rfc3339_utc();
        let after = OffsetDateTime::now_utc() + Duration::seconds(5);
        let parsed = OffsetDateTime::parse(&value, &Rfc3339).expect("parse UTC timestamp");

        assert!(value.ends_with('Z'));
        assert!(
            parsed >= before && parsed <= after,
            "expected {value} to be between {before} and {after}"
        );
    }
}
