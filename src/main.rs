use std::{fs, path::{Path, PathBuf}};

use crate::db::Database;

mod db;

#[tokio::main]
async fn main() {
    let db = Database::new_in_memory().await.unwrap();
    let path1 = Path::new("tmp/file.txt");
    let path2 = Path::new("report.xls");
    let path3 = Path::new("tmp/file.png");

    let id1 = db.add_file(&path1).await.unwrap();
    let id2 = db.add_file(&path2).await.unwrap();
    let id3 = db.add_file(&path3).await.unwrap();

    db.assign_tag_to_file("tag1", &path1).await.unwrap();
    db.assign_tag_to_file("tag1", &path3).await.unwrap();
    db.assign_tag_to_file("tag1", &path2).await.unwrap();

    let files = db.get_tag_files("tag1").await.unwrap();

    println!("{:?}", files);

}
