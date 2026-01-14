use std::{path::PathBuf, time::SystemTime};

#[derive(Debug, Clone)]
pub struct DBFile {
    pub id: i64,
    pub path: PathBuf,
    pub size: u64,
    pub content_hash: String,
    pub last_modified: SystemTime,
    pub last_accessed: SystemTime,
    pub created: SystemTime,
}

#[derive(Debug, Clone)]
pub struct DBTag {
    pub id: i64,
    pub name: String,
}

