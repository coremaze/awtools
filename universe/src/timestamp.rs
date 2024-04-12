use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn unix_epoch_timestamp_u32() -> u32 {
    u32::try_from(unix_epoch_timestamp_u64())
        // If we are past the max u32, the best we can do is report the latest possible time
        .unwrap_or(u32::MAX)
}

pub fn unix_epoch_timestamp_u64() -> u64 {
    SystemTime::now()
        // Time since unix epoch
        .duration_since(UNIX_EPOCH)
        .as_ref()
        // As seconds
        .map(Duration::as_secs)
        // If the time is before the unix epoch, the best we can give is 0
        .unwrap_or(0)
}

pub fn unix_epoch_timestamp_str() -> String {
    unix_epoch_timestamp_u64().to_string()
}
