mod db;
mod manager;
mod scanner;

pub use manager::{TagerManager, FileEntry, TagEntry};
use ::serde::Serialize;
use std::{cmp::Ordering, path::Path, time::{Duration, SystemTime, UNIX_EPOCH}};

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

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Image,
    Document,
    Video,
    Other,
    Directory,
}

pub fn get_entry_type(path: &Path) -> EntryType {
    if path.is_dir() {
        EntryType::Directory
    } else {
        match path.extension().and_then(|ext| ext.to_str()).map(|s| s.to_lowercase()) {
            Some(ext) if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext.as_str()) => EntryType::Image,
            Some(ext) if ["pdf", "doc", "docx", "txt", "md"].contains(&ext.as_str()) => EntryType::Document,
            Some(ext) if ["mp4", "mkv", "avi", "mov"].contains(&ext.as_str()) => EntryType::Video,
            _ => EntryType::Other,
        }
    }
}

#[cfg(test)]
mod tests;