use std::cmp::Ordering;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use tager::compare_system_times;
use tager::tm::TagerManager;
use tager::system_time_to_i64;

fn get_test_times() -> (SystemTime, SystemTime, SystemTime) {
        let now = SystemTime::now();
        let yesterday = now - Duration::from_secs(24 * 60 * 60);
        let last_week = now - Duration::from_secs(7 * 24 * 60 * 60);
        
        (now, yesterday, last_week)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    Ok(())
}
