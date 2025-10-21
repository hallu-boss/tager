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
}
