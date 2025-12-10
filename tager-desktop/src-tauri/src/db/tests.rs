use super::*;
//use tempfile::tempdir;

async fn setup_db() -> Database {
    Database::new_in_memory()
        .await
        .expect("Failed to create test database")
}

#[tokio::test]
async fn test_new_in_memory() {
    let db = Database::new_in_memory().await;
    assert!(db.is_ok());
}

// #[tokio::test]
// async fn test_new_file() {
//     let temp_dir = tempdir().expect("Failed to create temp dir");
//     let db_path = temp_dir.path().join("test.db");

//     let db = Database::new_file(&db_path).await;
//     assert!(db.is_ok());
//     assert!(db_path.exists());
// }

#[tokio::test]
async fn test_add_file() {
    let db = setup_db().await;

    let file_id = db.add_file("test/file.txt").await;
    assert!(file_id.is_ok());

    let id = file_id.unwrap();
    assert!(id > 0);
}

#[tokio::test]
async fn test_add_file_duplicate() {
    let db = setup_db().await;

    let id1 = db.add_file("test/file.txt").await.unwrap();
    let id2 = db.add_file("test/file.txt").await.unwrap();

    // Should return the same ID for duplicate paths
    assert_eq!(id1, id2);
}

#[tokio::test]
async fn test_add_file_if_not_exists() {
    let db = setup_db().await;

    let added1 = db.add_file_if_not_exists("test/file.txt").await.unwrap();
    assert!(added1, "First insert should return true");

    let added2 = db.add_file_if_not_exists("test/file.txt").await.unwrap();
    assert!(!added2, "Duplicate insert should return false");
}

#[tokio::test]
async fn test_file_exists() {
    let db = setup_db().await;

    let file_id = db.add_file("test/file.txt").await.unwrap();

    let exists = db.file_exists(file_id).await.unwrap();
    assert!(exists);

    let not_exists = db.file_exists(99999).await.unwrap();
    assert!(!not_exists);
}

#[tokio::test]
async fn test_assign_tag_to_file() {
    let db = setup_db().await;

    let file_id = db.add_file("test/file.txt").await.unwrap();

    let result = db.assign_tag_to_file_by_id(file_id, "important").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_assign_tag_to_nonexistent_file() {
    let db = setup_db().await;

    let result = db.assign_tag_to_file_by_id(99999, "important").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_assign_multiple_tags() {
    let db = setup_db().await;

    let file_id = db.add_file("test/file.txt").await.unwrap();

    db.assign_tag_to_file_by_id(file_id, "important")
        .await
        .unwrap();
    db.assign_tag_to_file_by_id(file_id, "work").await.unwrap();
    db.assign_tag_to_file_by_id(file_id, "urgent")
        .await
        .unwrap();

    let tags = db.get_tags_for_file("test/file.txt").await.unwrap();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"important".to_string()));
    assert!(tags.contains(&"work".to_string()));
    assert!(tags.contains(&"urgent".to_string()));
}

#[tokio::test]
async fn test_assign_duplicate_tag() {
    let db = setup_db().await;

    let file_id = db.add_file("test/file.txt").await.unwrap();

    db.assign_tag_to_file_by_id(file_id, "important")
        .await
        .unwrap();
    db.assign_tag_to_file_by_id(file_id, "important")
        .await
        .unwrap();

    let tags = db.get_tags_for_file("test/file.txt").await.unwrap();
    assert_eq!(tags.len(), 1, "Duplicate tags should not be added");
}

#[tokio::test]
async fn test_get_tags_for_file() {
    let db = setup_db().await;

    let file_id = db.add_file("test/file.txt").await.unwrap();
    db.assign_tag_to_file_by_id(file_id, "tag1").await.unwrap();
    db.assign_tag_to_file_by_id(file_id, "tag2").await.unwrap();

    let tags = db.get_tags_for_file("test/file.txt").await.unwrap();
    assert_eq!(tags.len(), 2);
}

#[tokio::test]
async fn test_get_tags_for_nonexistent_file() {
    let db = setup_db().await;

    let tags = db.get_tags_for_file("nonexistent.txt").await.unwrap();
    assert_eq!(tags.len(), 0);
}

#[tokio::test]
async fn test_get_untagged_files() {
    let db = setup_db().await;

    let file1_id = db.add_file("tagged.txt").await.unwrap();
    db.add_file("untagged1.txt").await.unwrap();
    db.add_file("untagged2.txt").await.unwrap();

    db.assign_tag_to_file_by_id(file1_id, "important")
        .await
        .unwrap();

    let untagged = db.get_untagged_files(None).await.unwrap();
    assert_eq!(untagged.len(), 2);

    let paths: Vec<String> = untagged.iter().map(|(_, p)| p.clone()).collect();
    assert!(paths.contains(&"untagged1.txt".to_string()));
    assert!(paths.contains(&"untagged2.txt".to_string()));
    assert!(!paths.contains(&"tagged.txt".to_string()));
}

#[tokio::test]
async fn test_get_untagged_files_ordered_by_id() {
    let db = setup_db().await;

    db.add_file("file3.txt").await.unwrap();
    db.add_file("file1.txt").await.unwrap();
    db.add_file("file2.txt").await.unwrap();

    let untagged = db.get_untagged_files(Some(FilesOrderBy::Id)).await.unwrap();

    // IDs should be in ascending order
    let ids: Vec<i64> = untagged.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_get_untagged_files_ordered_by_path() {
    let db = setup_db().await;

    db.add_file("file3.txt").await.unwrap();
    db.add_file("file1.txt").await.unwrap();
    db.add_file("file2.txt").await.unwrap();

    let untagged = db
        .get_untagged_files(Some(FilesOrderBy::Path))
        .await
        .unwrap();

    let paths: Vec<String> = untagged.iter().map(|(_, p)| p.clone()).collect();
    assert_eq!(paths, vec!["file1.txt", "file2.txt", "file3.txt"]);
}

#[tokio::test]
async fn test_get_files_for_tag() {
    let db = setup_db().await;

    let file1_id = db.add_file("file1.txt").await.unwrap();
    let file2_id = db.add_file("file2.txt").await.unwrap();
    let file3_id = db.add_file("file3.txt").await.unwrap();

    db.assign_tag_to_file_by_id(file1_id, "important")
        .await
        .unwrap();
    db.assign_tag_to_file_by_id(file2_id, "important")
        .await
        .unwrap();
    db.assign_tag_to_file_by_id(file3_id, "other")
        .await
        .unwrap();

    let files = db.get_files_for_tag("important", None).await.unwrap();
    assert_eq!(files.len(), 2);

    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    assert!(paths.contains(&"file1.txt".to_string()));
    assert!(paths.contains(&"file2.txt".to_string()));
}

#[tokio::test]
async fn test_get_files_for_tag_with_multiple_tags() {
    let db = setup_db().await;

    let file_id = db.add_file("file.txt").await.unwrap();
    db.assign_tag_to_file_by_id(file_id, "important")
        .await
        .unwrap();
    db.assign_tag_to_file_by_id(file_id, "work").await.unwrap();
    db.assign_tag_to_file_by_id(file_id, "urgent")
        .await
        .unwrap();

    let files = db.get_files_for_tag("work", None).await.unwrap();
    assert_eq!(files.len(), 1);

    let file = &files[0];
    assert_eq!(file.path, "file.txt");
    assert_eq!(file.tags.len(), 3);
    assert!(file.tags.contains(&"important".to_string()));
    assert!(file.tags.contains(&"work".to_string()));
    assert!(file.tags.contains(&"urgent".to_string()));
}

#[tokio::test]
async fn test_get_files_for_tag_ordered_by_path() {
    let db = setup_db().await;

    let file1_id = db.add_file("zebra.txt").await.unwrap();
    let file2_id = db.add_file("alpha.txt").await.unwrap();
    let file3_id = db.add_file("beta.txt").await.unwrap();

    db.assign_tag_to_file_by_id(file1_id, "tag").await.unwrap();
    db.assign_tag_to_file_by_id(file2_id, "tag").await.unwrap();
    db.assign_tag_to_file_by_id(file3_id, "tag").await.unwrap();

    let files = db
        .get_files_for_tag("tag", Some(FilesOrderBy::Path))
        .await
        .unwrap();

    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    assert_eq!(paths, vec!["alpha.txt", "beta.txt", "zebra.txt"]);
}

#[tokio::test]
async fn test_get_files_for_nonexistent_tag() {
    let db = setup_db().await;

    db.add_file("file.txt").await.unwrap();

    let files = db.get_files_for_tag("nonexistent", None).await.unwrap();
    assert_eq!(files.len(), 0);
}

#[tokio::test]
async fn test_tags_are_shared_across_files() {
    let db = setup_db().await;

    let file1_id = db.add_file("file1.txt").await.unwrap();
    let file2_id = db.add_file("file2.txt").await.unwrap();

    db.assign_tag_to_file_by_id(file1_id, "shared")
        .await
        .unwrap();
    db.assign_tag_to_file_by_id(file2_id, "shared")
        .await
        .unwrap();

    let files = db.get_files_for_tag("shared", None).await.unwrap();
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn test_file_with_tags_struct() {
    let db = setup_db().await;

    let file_id = db.add_file("test.txt").await.unwrap();
    db.assign_tag_to_file_by_id(file_id, "tag1").await.unwrap();
    db.assign_tag_to_file_by_id(file_id, "tag2").await.unwrap();

    let files = db.get_files_for_tag("tag1", None).await.unwrap();
    let file = &files[0];

    assert_eq!(file.id, file_id);
    assert_eq!(file.path, "test.txt");
    assert_eq!(file.tags.len(), 2);
}

#[tokio::test]
async fn test_complex_tagging_scenario() {
    let db = setup_db().await;

    // Create multiple files with overlapping tags
    let work1_id = db.add_file("work/report.txt").await.unwrap();
    let work2_id = db.add_file("work/notes.txt").await.unwrap();
    let personal_id = db.add_file("personal/diary.txt").await.unwrap();
    let mixed_id = db.add_file("mixed.txt").await.unwrap();

    db.assign_tag_to_file_by_id(work1_id, "work").await.unwrap();
    db.assign_tag_to_file_by_id(work1_id, "important")
        .await
        .unwrap();

    db.assign_tag_to_file_by_id(work2_id, "work").await.unwrap();

    db.assign_tag_to_file_by_id(personal_id, "personal")
        .await
        .unwrap();

    db.assign_tag_to_file_by_id(mixed_id, "work").await.unwrap();
    db.assign_tag_to_file_by_id(mixed_id, "personal")
        .await
        .unwrap();
    db.assign_tag_to_file_by_id(mixed_id, "important")
        .await
        .unwrap();

    // Test work tag
    let work_files = db.get_files_for_tag("work", None).await.unwrap();
    assert_eq!(work_files.len(), 3);

    // Test important tag
    let important_files = db.get_files_for_tag("important", None).await.unwrap();
    assert_eq!(important_files.len(), 2);

    // Test personal tag
    let personal_files = db.get_files_for_tag("personal", None).await.unwrap();
    assert_eq!(personal_files.len(), 2);

    // Test untagged
    db.add_file("untagged.txt").await.unwrap();
    let untagged = db.get_untagged_files(None).await.unwrap();
    assert_eq!(untagged.len(), 1);
}

#[tokio::test]
async fn test_pool_access() {
    let db = setup_db().await;
    let pool = db.pool();

    // Verify we can execute queries directly on the pool
    let result = sqlx::query("SELECT 1 as test").fetch_one(pool).await;

    assert!(result.is_ok());
}
