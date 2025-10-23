use super::*;
use sqlx::Row;

#[tokio::test]
async fn test_add_file() {
    let db = Database::new_in_memory().await.unwrap();
    let path = Path::new("tmp/file.txt");

    let file_id = db.add_file(&path).await.unwrap();

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

    let path = Path::new("tmp/file.png");
    let file_id = db.add_file(&path).await.unwrap();

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

    let path = Path::new("file.md");
    let file_id = db.add_file(&path).await.unwrap();

    assert!(file_id == 3);

    let row = sqlx::query("SELECT id, path FROM files WHERE id = ?")
        .bind(file_id)
        .fetch_one(&db.pool)
        .await
        .unwrap();

    let id: i64 = row.get("id");
    let path: String = row.get("path");

    assert_eq!(id, file_id);
    assert_eq!(path, "file.md");
}

#[tokio::test]
async fn test_remove_file() {
    let db = Database::new_in_memory().await.unwrap();
    let path = Path::new("tmp/file.txt");
    let path2 = Path::new("tmp/file.png");

    db.add_file(&path).await.unwrap();
    db.add_file(&path2).await.unwrap();

    let affected = db.remove_file(&path2).await.unwrap();

    assert!(affected == 1);

    let res = sqlx::query("SELECT * FROM files WHERE path = ?")
        .bind(path2.to_string_lossy())
        .fetch_one(&db.pool)
        .await;

    assert!(res.is_err());

    let res = sqlx::query("SELECT * FROM files WHERE path = ?")
        .bind(path.to_string_lossy())
        .fetch_one(&db.pool)
        .await;

    assert!(res.is_ok())

}

#[tokio::test]
async fn test_assign_tag_to_file() {
    let db = Database::new_in_memory().await.unwrap();
    let path = Path::new("tmp/file.txt");

    db.add_file(&path).await.unwrap();

    let res = db.assign_tag_to_file("tag", &path).await;
    assert!(res.is_ok());

    let res = sqlx::query("SELECT * FROM file_tags WHERE file_id = 1 AND tag_id = 1")
        .fetch_one(&db.pool)
        .await;

    assert!(res.is_ok())
}

#[tokio::test]
async fn test_get_file_tags() {
    let db = Database::new_in_memory().await.unwrap();
    let path = Path::new("tmp/file.txt");

    db.add_file(&path).await.unwrap();

    db.assign_tag_to_file("tag1", &path).await.unwrap();
    db.assign_tag_to_file("tag2", &path).await.unwrap();
    db.assign_tag_to_file("tag3", &path).await.unwrap();

    let tags = db.get_file_tags(&path).await.unwrap();

    assert!(tags.iter().any(|t| t == "tag1"));
    assert!(tags.iter().any(|t| t == "tag2"));
    assert!(tags.iter().any(|t| t == "tag3"));
}

#[tokio::test]
async fn test_get_tag_files() {
    // Utwórz bazę w pamięci z FK włączonymi
    let db = Database::new_in_memory().await.unwrap();

    // Zdefiniuj pliki w "tmp/"
    let path1 = Path::new("tmp/file.txt");
    let path2 = Path::new("tmp/report.xls");
    let path3 = Path::new("tmp/file.png");
    let path4 = Path::new("tmp/image.png");

    // Dodaj pliki do bazy
    let id1 = db.add_file(&path1).await.unwrap();
    let id2 = db.add_file(&path2).await.unwrap();
    let id3 = db.add_file(&path3).await.unwrap();
    let id4 = db.add_file(&path4).await.unwrap();

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
    assert_eq!(id4, 4);

    // Przypisz jeden tag do wszystkich plików
    db.assign_tag_to_file("tag1", &path1).await.unwrap();
    db.assign_tag_to_file("tag1", &path2).await.unwrap();
    db.assign_tag_to_file("tag1", &path3).await.unwrap();
    db.assign_tag_to_file("tag1", &path4).await.unwrap();

    // Pobierz pliki dla tagu "tag1"
    let files = db.get_tag_files("tag1").await.unwrap();

    // Sprawdź, że wszystkie pliki są w wyniku
    assert!(files.iter().any(|t| t == "tmp/file.txt"));
    assert!(files.iter().any(|t| t == "tmp/report.xls"));
    assert!(files.iter().any(|t| t == "tmp/file.png"));
    assert!(files.iter().any(|t| t == "tmp/image.png"));

    // Dodatkowo sprawdź, że liczba plików zgadza się z ilością wstawionych
    assert_eq!(files.len(), 4);
}
