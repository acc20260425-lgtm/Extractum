pub(crate) fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub(crate) fn now_rfc3339_utc() -> String {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    OffsetDateTime::from_unix_timestamp(now_secs())
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .format(&Rfc3339)
        .expect("format current UTC timestamp")
}

pub(crate) fn ymd_to_unix_midnight(value: &str) -> Option<i64> {
    let value = value.trim();
    let normalized = if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        format!("{}-{}-{}", &value[0..4], &value[4..6], &value[6..8])
    } else {
        value.to_string()
    };

    let mut parts = normalized.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() || !is_valid_ymd(year, month, day) {
        return None;
    }

    Some(days_from_civil(year, month, day) * 86_400)
}

fn is_valid_ymd(year: i64, month: i64, day: i64) -> bool {
    (1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
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
