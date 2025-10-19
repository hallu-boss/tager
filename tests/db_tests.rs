use tager::db::Database;

#[tokio::test]
async fn test_assign_tag_to_file_creates_records() {
    let db = Database::new_in_memory().await.unwrap();

    db.assign_tag_to_file("photo1.jpg", "morze").await.unwrap();
    db.assign_tag_to_file("photo1.jpg", "zachód").await.unwrap();

    let tags = db.get_tags_for_file("photo1.jpg").await.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"morze".to_string()));
    assert!(tags.contains(&"zachód".to_string()));
}

#[tokio::test]
async fn test_assign_tag_to_same_file_and_tag_twice_is_idempotent() {
    let db = Database::new_in_memory().await.unwrap();

    db.assign_tag_to_file("photo2.jpg", "las").await.unwrap();
    db.assign_tag_to_file("photo2.jpg", "las").await.unwrap(); // drugi raz

    let tags = db.get_tags_for_file("photo2.jpg").await.unwrap();
    assert_eq!(tags.len(), 1); // nie dubluje się
    assert_eq!(tags[0], "las");
}
