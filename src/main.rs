
mod db;
use db::Database;

#[tokio::main]
async fn main() {
    let db = Database::from_file().await.unwrap();

    let tag1 = String::from("important");
    let tag2 = String::from("important2");

    let file_id = db.add_file("tmp/file.txt").await.unwrap();
    let tag_id1 = db.add_tag(&tag1).await.unwrap();
    let tag_id2 = db.add_tag(&tag2).await.unwrap();

    db.assign_tag_to_file( tag_id1, file_id).await;
    db.assign_tag_to_file( tag_id2, file_id).await;

    let files = db.get_all_files().await.unwrap();
    println!("{:?}", files);
}
