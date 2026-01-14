use std::{cmp::Ordering, time::{Duration, SystemTime, UNIX_EPOCH}};

pub mod tm;

// Funkcje konwersji
pub fn system_time_to_i64(st: SystemTime) -> i64 {
    st.duration_since(UNIX_EPOCH)
        .expect("czas przed 1970?")
        .as_secs() as i64
}

pub fn i64_to_system_time(ts: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(ts as u64)
}

pub fn compare_system_times(a: SystemTime, b: SystemTime) -> Ordering {
    let a_ts = system_time_to_i64(a);
    let b_ts = system_time_to_i64(b);

    a_ts.cmp(&b_ts)
}