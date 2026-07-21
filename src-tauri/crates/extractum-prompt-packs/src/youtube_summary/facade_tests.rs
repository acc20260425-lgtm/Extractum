use super::now_string;

#[test]
fn now_string_uses_current_utc_time() {
    use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

    let before = OffsetDateTime::now_utc() - Duration::seconds(5);
    let value = now_string();
    let after = OffsetDateTime::now_utc() + Duration::seconds(5);
    let parsed = OffsetDateTime::parse(&value, &Rfc3339).expect("parse youtube summary timestamp");

    assert_ne!(value, "2026-06-14T00:00:00Z");
    assert!(
        parsed >= before && parsed <= after,
        "expected {value} to be between {before} and {after}"
    );
}
