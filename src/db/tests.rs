use super::*;
use sqlx::Row;

#[tokio::test]
async fn new_in_memory_allows_simple_query_or_skips_on_error() {
    let db = Database::new_in_memory().await.unwrap();

    let v: i64 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&db.pool)
        .await
        .expect("SELECT 1 should succeed");
    assert_eq!(v, 1);
}

#[tokio::test]
async fn test_schema_initialization() {
    let db = Database::new_in_memory().await.unwrap();

    let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&db.pool)
        .await
        .unwrap();

    let table_names: Vec<String> = rows.iter().map(|r| r.get::<String, _>("name")).collect();

    assert!(
        table_names.contains(&"files".to_string()),
        "Table 'files' not found in schema"
    );
    assert!(
        table_names.contains(&"tags".to_string()),
        "Table 'tags' not found in schema"
    );
    assert!(
        table_names.contains(&"file_tags".to_string()),
        "Table 'file_tags' not found in schema"
    );
}

#[tokio::test]
async fn test_add_file() {
    let db = Database::new_in_memory().await.unwrap();

    let file_id = db.add_file("tmp/file.txt").await.unwrap();

    assert!(file_id == 1);

    let row = sqlx::query("SELECT id, path FROM files WHERE id = ?")
        .bind(file_id)
        .fetch_one(&db.pool)
        .await
        .unwrap();

    let id: i64 = row.get("id");
    let path: String = row.get("path");

    assert_eq!(id, file_id);
    assert_eq!(path, "tmp/file.txt");

    let file_id = db.add_file("tmp/file.png").await.unwrap();

    assert!(file_id == 2);

    let row = sqlx::query("SELECT id, path FROM files WHERE id = ?")
        .bind(file_id)
        .fetch_one(&db.pool)
        .await
        .unwrap();

    let id: i64 = row.get("id");
    let path: String = row.get("path");

    assert_eq!(id, file_id);
    assert_eq!(path, "tmp/file.png");
}

#[tokio::test]
async fn test_add_tag() {
    let db = Database::new_in_memory().await.unwrap();

    let tag_id = db.add_tag("test").await.unwrap();

    assert!(tag_id > 0);

    let row = sqlx::query("SELECT id, name FROM tags WHERE id = ?")
        .bind(tag_id)
        .fetch_one(&db.pool)
        .await
        .unwrap();

    let id: i64 = row.get("id");
    let name: String = row.get("name");

    assert_eq!(id, tag_id);
    assert_eq!(name, "test");

    let tag_id = db.add_tag("sea").await.unwrap();

    assert!(tag_id == 2);

    let row = sqlx::query("SELECT id, name FROM tags WHERE id = ?")
        .bind(tag_id)
        .fetch_one(&db.pool)
        .await
        .unwrap();

    let id: i64 = row.get("id");
    let name: String = row.get("name");

    assert_eq!(id, tag_id);
    assert_eq!(name, "sea");
}

#[tokio::test]
async fn test_assign_tag_to_file() {
    let db = Database::new_in_memory().await.unwrap();

    let file_id = db.add_file("tmp/file.txt").await.unwrap();
    let tag_id = db.add_tag("important").await.unwrap();

    // przypisanie
    let result = db.assign_tag_to_file(tag_id, file_id).await;
    assert!(result.is_ok(), "assign_tag_to_file() powinno się powieść");

    // weryfikacja wpisu
    let row = sqlx::query("SELECT file_id, tag_id FROM file_tags WHERE file_id = ? AND tag_id = ?")
        .bind(file_id)
        .bind(tag_id)
        .fetch_one(&db.pool)
        .await
        .unwrap();

    let db_file_id: i64 = row.get("file_id");
    let db_tag_id: i64 = row.get("tag_id");

    assert_eq!(db_file_id, file_id);
    assert_eq!(db_tag_id, tag_id);
}

#[tokio::test]
async fn test_get_file_tags() {
    let db = Database::new_in_memory().await.unwrap();

    let file_id = db.add_file("tmp/file.txt").await.unwrap();
    let tag_id = db.add_tag("important").await.unwrap();

    assert!(db.assign_tag_to_file(tag_id, file_id).await.is_ok());

    let tags = db.get_file_tags(file_id).await.unwrap();

    assert!(tags[0] == "important")
}

#[tokio::test]
async fn test_get_all_files() {
    let db = Database::new_in_memory().await.unwrap();

    let tag1 = String::from("important");
    let tag2 = String::from("important2");

    let file_id = db.add_file("tmp/file.txt").await.unwrap();
    let tag_id = db.add_tag(&tag1).await.unwrap();
    let tag_id = db.add_tag(&tag2).await.unwrap();

    let files = db.get_all_files().await.unwrap();
    assert!(files[0].path == "tmp/file.xt");
    assert!(files[0].tags.contains(&tag1));
    assert!(files[0].tags.contains(&tag2));
}
