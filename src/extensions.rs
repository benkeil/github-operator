use std::time::Duration;

pub trait DurationExtension {
    fn from_minutes(minutes: u64) -> Duration;
}

impl DurationExtension for Duration {
    fn from_minutes(minutes: u64) -> Duration {
        Duration::from_secs(minutes * 60)
    }
}
