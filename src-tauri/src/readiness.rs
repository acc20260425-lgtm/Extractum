pub(crate) type ModelVersion = i64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReadinessStatus {
    NeverBuilt,
    Building,
    Ready,
    Stale,
    Failed,
}

impl ReadinessStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NeverBuilt => "never_built",
            Self::Building => "building",
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Failed => "failed",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "never_built" => Some(Self::NeverBuilt),
            "building" => Some(Self::Building),
            "ready" => Some(Self::Ready),
            "stale" => Some(Self::Stale),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

pub(crate) fn is_ready_current(
    status: ReadinessStatus,
    found_version: ModelVersion,
    current_version: ModelVersion,
) -> bool {
    status == ReadinessStatus::Ready && found_version == current_version
}

pub(crate) fn mark_stale(status: ReadinessStatus) -> ReadinessStatus {
    match status {
        ReadinessStatus::Ready => ReadinessStatus::Stale,
        other => other,
    }
}

pub(crate) fn mark_failed() -> ReadinessStatus {
    ReadinessStatus::Failed
}

#[cfg(test)]
mod tests {
    use super::{is_ready_current, mark_failed, mark_stale, ModelVersion, ReadinessStatus};

    #[test]
    fn readiness_status_roundtrips_wire_values() {
        for (status, wire) in [
            (ReadinessStatus::NeverBuilt, "never_built"),
            (ReadinessStatus::Building, "building"),
            (ReadinessStatus::Ready, "ready"),
            (ReadinessStatus::Stale, "stale"),
            (ReadinessStatus::Failed, "failed"),
        ] {
            assert_eq!(status.as_str(), wire);
            assert_eq!(ReadinessStatus::parse(wire), Some(status));
        }
        assert_eq!(ReadinessStatus::parse("unknown"), None);
    }

    #[test]
    fn is_ready_current_requires_ready_status_and_current_version() {
        const CURRENT: ModelVersion = 3;

        assert!(is_ready_current(ReadinessStatus::Ready, CURRENT, CURRENT));
        assert!(!is_ready_current(
            ReadinessStatus::Ready,
            CURRENT - 1,
            CURRENT
        ));
        assert!(!is_ready_current(ReadinessStatus::Stale, CURRENT, CURRENT));
        assert!(!is_ready_current(ReadinessStatus::Failed, CURRENT, CURRENT));
        assert!(!is_ready_current(
            ReadinessStatus::Building,
            CURRENT,
            CURRENT
        ));
        assert!(!is_ready_current(
            ReadinessStatus::NeverBuilt,
            CURRENT,
            CURRENT
        ));
    }

    #[test]
    fn mark_stale_only_changes_ready_state() {
        assert_eq!(mark_stale(ReadinessStatus::Ready), ReadinessStatus::Stale);
        assert_eq!(
            mark_stale(ReadinessStatus::Building),
            ReadinessStatus::Building
        );
        assert_eq!(mark_stale(ReadinessStatus::Failed), ReadinessStatus::Failed);
        assert_eq!(
            mark_stale(ReadinessStatus::NeverBuilt),
            ReadinessStatus::NeverBuilt
        );
    }

    #[test]
    fn mark_failed_returns_failed_state() {
        assert_eq!(mark_failed(), ReadinessStatus::Failed);
    }
}
