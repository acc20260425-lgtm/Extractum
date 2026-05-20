use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug)]
pub(crate) struct ActiveJobGuards<K> {
    active_by_key: HashMap<K, String>,
    key_by_job_id: HashMap<String, K>,
}

impl<K> Default for ActiveJobGuards<K> {
    fn default() -> Self {
        Self {
            active_by_key: HashMap::new(),
            key_by_job_id: HashMap::new(),
        }
    }
}

impl<K> ActiveJobGuards<K>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn active_job_id(&self, key: &K) -> Option<&str> {
        self.active_by_key.get(key).map(String::as_str)
    }

    pub(crate) fn track(&mut self, key: K, job_id: String) {
        self.active_by_key.insert(key.clone(), job_id.clone());
        self.key_by_job_id.insert(job_id, key);
    }

    pub(crate) fn release_by_job_id(&mut self, job_id: &str) -> Option<K> {
        let key = self.key_by_job_id.remove(job_id)?;
        self.active_by_key.remove(&key);
        Some(key)
    }
}

#[derive(Debug, Default)]
pub(crate) struct CancellationState {
    requested: HashSet<String>,
}

impl CancellationState {
    pub(crate) fn request(&mut self, job_id: impl Into<String>) {
        self.requested.insert(job_id.into());
    }

    pub(crate) fn is_requested(&self, job_id: &str) -> bool {
        self.requested.contains(job_id)
    }

    pub(crate) fn clear(&mut self, job_id: &str) -> bool {
        self.requested.remove(job_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{ActiveJobGuards, CancellationState};

    #[test]
    fn active_job_guards_track_and_release_scoped_jobs() {
        let mut guards = ActiveJobGuards::default();

        assert_eq!(guards.active_job_id(&7), None);

        guards.track(7, "job-1".to_string());

        assert_eq!(guards.active_job_id(&7), Some("job-1"));
        assert_eq!(guards.release_by_job_id("job-1"), Some(7));
        assert_eq!(guards.active_job_id(&7), None);
        assert_eq!(guards.release_by_job_id("job-1"), None);
    }

    #[test]
    fn cancellation_state_marks_checks_and_clears_jobs() {
        let mut cancellation = CancellationState::default();

        assert!(!cancellation.is_requested("job-1"));

        cancellation.request("job-1");

        assert!(cancellation.is_requested("job-1"));
        assert!(cancellation.clear("job-1"));
        assert!(!cancellation.is_requested("job-1"));
        assert!(!cancellation.clear("job-1"));
    }
}
