use std::time::Duration;

pub type Milliseconds = u64;

pub fn ms_to_dur(ms: Milliseconds) -> Duration {
    Duration::from_millis(ms)
}
