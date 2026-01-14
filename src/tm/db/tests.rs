use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use crate::{compare_system_times, tm::db::Database};

fn get_test_times() -> (SystemTime, SystemTime, SystemTime) {
    let now = SystemTime::now();
    let yesterday = now - Duration::from_secs(24 * 60 * 60);
    let last_week = now - Duration::from_secs(7 * 24 * 60 * 60);

    (now, yesterday, last_week)
}

#[tokio::test]
async fn test_update_file() {
    let db = Database::new_in_memory().await.unwrap();
    let path = PathBuf::from("/test/file.txt");
    let (last_modified, last_accessed, created) = get_test_times();

    let id = db
        .create_file(
            path.clone(),
            1024,
            "abc123".to_string(),
            last_modified,
            last_accessed,
            created,
        )
        .await
        .unwrap();

    // Aktualizuj tylko rozmiar
    let updated = db
        .update_file(id, None, Some(2048), None, None, None, None)
        .await
        .unwrap();

    assert!(updated);

    let file = db.get_file(id).await.unwrap().unwrap();
    assert_eq!(file.path, path); // Nie zmienione
    assert_eq!(file.size, 2048); // Zmienione
    assert_eq!(file.content_hash, "abc123"); // Nie zmienione
}

#[tokio::test]
async fn test_update_file_multiple_fields() {
    let db = Database::new_in_memory().await.unwrap();
    let path = PathBuf::from("/test/file.txt");
    let (last_modified, last_accessed, created) = get_test_times();

    let id = db
        .create_file(
            path,
            1024,
            "old_hash".to_string(),
            last_modified,
            last_accessed,
            created,
        )
        .await
        .unwrap();

    // Aktualizuj kilka pól
    let new_last_modified = SystemTime::now();
    let new_created = SystemTime::now();

    let updated = db
        .update_file(
            id,
            Some(PathBuf::from("/test/new_path.txt")),
            Some(2048),
            Some("new_hash".to_string()),
            Some(new_last_modified),
            None, // last_accessed nie zmienione
            Some(new_created),
        )
        .await
        .unwrap();

    assert!(updated);

    let file = db.get_file(id).await.unwrap().unwrap();
    assert_eq!(file.path, PathBuf::from("/test/new_path.txt"));
    assert_eq!(file.size, 2048);
    assert_eq!(file.content_hash, "new_hash");
    assert!(compare_system_times(file.last_modified, new_last_modified).is_eq());
    assert!(compare_system_times(file.created, new_created).is_eq());
}

#[tokio::test]
async fn test_create_and_get_file() {
    let db = Database::new_in_memory().await.unwrap();
    let (last_modified, last_accessed, created) = get_test_times();

    let id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "abc123".to_string(),
            last_modified,
            last_accessed,
            created,
        )
        .await
        .unwrap();

    assert!(id > 0);

    let file = db.get_file(id).await.unwrap().unwrap();
    assert_eq!(file.path, PathBuf::from("/test/file.txt"));
    assert_eq!(file.size, 1024);
    assert_eq!(file.content_hash, "abc123");
}

#[tokio::test]
async fn test_get_nonexistent_file() {
    let db = Database::new_in_memory().await.unwrap();
    let result = db.get_file(999).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_all_files() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz kilka plików
    for i in 0..5 {
        db.create_file(
            PathBuf::from(format!("/test/file{}.txt", i)),
            100 + i as u64,
            format!("hash{}", i),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();
    }

    let files = db.get_all_files(None, None).await.unwrap();
    assert_eq!(files.len(), 5);

    // Test paginacji
    let limited = db.get_all_files(Some(2), None).await.unwrap();
    assert_eq!(limited.len(), 2);

    let offset = db.get_all_files(Some(2), Some(2)).await.unwrap();
    assert_eq!(offset.len(), 2);
}

#[tokio::test]
async fn test_get_files_by_path() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz kilka plików z tą samą ścieżką (różne wersje)
    let path = PathBuf::from("/test/file.txt");
    for i in 0..3 {
        let modified = lm + Duration::from_secs(i as u64);
        db.create_file(
            path.clone(),
            100 + i as u64,
            format!("hash{}", i),
            modified,
            la,
            cr,
        )
        .await
        .unwrap();
    }

    let files = db.get_files_by_path(path.clone()).await.unwrap();
    assert_eq!(files.len(), 3);

    // Sprawdź czy są posortowane malejąco po last_modified
    assert!(files[0].last_modified >= files[1].last_modified);
    assert!(files[1].last_modified >= files[2].last_modified);
}

#[tokio::test]
async fn test_delete_file() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "abc123".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    // Usuń plik
    let deleted = db.delete_file(id).await.unwrap();
    assert!(deleted);

    // Sprawdź czy plik został usunięty
    let file = db.get_file(id).await.unwrap();
    assert!(file.is_none());

    // Spróbuj usunąć nieistniejący plik
    let deleted = db.delete_file(999).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_delete_files_by_path() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz kilka plików z różnymi ścieżkami
    db.create_file(
        PathBuf::from("/test/file1.txt"),
        100,
        "hash1".to_string(),
        lm,
        la,
        cr,
    )
    .await
    .unwrap();

    db.create_file(
        PathBuf::from("/test/file2.txt"),
        200,
        "hash2".to_string(),
        lm,
        la,
        cr,
    )
    .await
    .unwrap();

    // Utwórz 2 pliki z tą samą ścieżką
    let path_to_delete = PathBuf::from("/test/duplicate.txt");
    db.create_file(path_to_delete.clone(), 300, "hash3".to_string(), lm, la, cr)
        .await
        .unwrap();

    let modified = lm + Duration::from_secs(10);
    db.create_file(
        path_to_delete.clone(),
        400,
        "hash4".to_string(),
        modified,
        la,
        cr,
    )
    .await
    .unwrap();

    // Usuń wszystkie pliki o danej ścieżce
    let deleted_count = db.delete_files_by_path(path_to_delete).await.unwrap();
    assert_eq!(deleted_count, 2);

    // Sprawdź czy pozostały tylko 2 pliki
    let files = db.get_all_files(None, None).await.unwrap();
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn test_file_exists() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "abc123".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    assert!(db.file_exists(id).await.unwrap());
    assert!(!db.file_exists(999).await.unwrap());
}

#[tokio::test]
async fn test_get_files_by_hash() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let common_hash = "duplicate_hash".to_string();

    // Utwórz 2 pliki z tym samym hashem
    db.create_file(
        PathBuf::from("/test/file1.txt"),
        100,
        common_hash.clone(),
        lm,
        la,
        cr,
    )
    .await
    .unwrap();

    db.create_file(
        PathBuf::from("/test/file2.txt"),
        200,
        common_hash.clone(),
        lm,
        la,
        cr,
    )
    .await
    .unwrap();

    // Plik z innym hashem
    db.create_file(
        PathBuf::from("/test/unique.txt"),
        300,
        "unique_hash".to_string(),
        lm,
        la,
        cr,
    )
    .await
    .unwrap();

    let duplicates = db.get_files_by_hash(&common_hash).await.unwrap();
    assert_eq!(duplicates.len(), 2);

    let no_duplicates = db.get_files_by_hash("nonexistent").await.unwrap();
    assert!(no_duplicates.is_empty());
}

#[tokio::test]
async fn test_count_files() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Początkowo 0 plików
    let count = db.count_files().await.unwrap();
    assert_eq!(count, 0);

    // Dodaj 3 pliki
    for i in 0..3 {
        db.create_file(
            PathBuf::from(format!("/test/file{}.txt", i)),
            100,
            format!("hash{}", i),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();
    }

    let count = db.count_files().await.unwrap();
    assert_eq!(count, 3);
}

// ========== TESTOWANIE OPERACJI NA TAGACH ==========

#[tokio::test]
async fn test_create_and_get_tag() {
    let db = Database::new_in_memory().await.unwrap();

    let id = db.create_tag("important").await.unwrap();
    assert!(id > 0);

    let tag = db.get_tag(id).await.unwrap().unwrap();
    assert_eq!(tag.name, "important");

    let tag_by_name = db.get_tag_by_name("important").await.unwrap().unwrap();
    assert_eq!(tag_by_name.id, id);
    assert_eq!(tag_by_name.name, "important");
}

#[tokio::test]
async fn test_create_duplicate_tag() {
    let db = Database::new_in_memory().await.unwrap();

    let id1 = db.create_tag("work").await.unwrap();
    let id2 = db.create_tag("work").await.unwrap(); // Powinien zwrócić istniejący ID

    assert_eq!(id1, id2);

    let tags = db.get_all_tags().await.unwrap();
    assert_eq!(tags.len(), 1); // Tylko jeden tag
}

#[tokio::test]
async fn test_get_tag_id_by_name() {
    let db = Database::new_in_memory().await.unwrap();

    let id = db.create_tag("project").await.unwrap();

    let found_id = db.get_tag_id_by_name("project").await.unwrap().unwrap();
    assert_eq!(found_id, id);

    let not_found = db.get_tag_id_by_name("nonexistent").await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_get_all_tags() {
    let db = Database::new_in_memory().await.unwrap();

    // Utwórz kilka tagów
    let tag_names = vec!["work", "personal", "urgent", "archive"];
    for name in &tag_names {
        db.create_tag(name).await.unwrap();
    }

    let tags = db.get_all_tags().await.unwrap();
    assert_eq!(tags.len(), 4);

    // Sprawdź czy są posortowane alfabetycznie
    let tag_names_sorted: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        tag_names_sorted,
        vec!["archive", "personal", "urgent", "work"]
    );
}

#[tokio::test]
async fn test_update_tag() {
    let db = Database::new_in_memory().await.unwrap();

    let id = db.create_tag("oldname").await.unwrap();

    // Zmień nazwę
    let updated = db.update_tag(id, "newname").await.unwrap();
    assert!(updated);

    let tag = db.get_tag(id).await.unwrap().unwrap();
    assert_eq!(tag.name, "newname");

    // Spróbuj zmienić na istniejącą nazwę (powinno się nie udać)
    let _ = db.create_tag("conflict").await.unwrap();
    let result = db.update_tag(id, "conflict").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_nonexistent_tag() {
    let db = Database::new_in_memory().await.unwrap();

    let updated = db.update_tag(999, "newname").await.unwrap();
    assert!(!updated); // Tag nie istnieje
}

#[tokio::test]
async fn test_delete_tag() {
    let db = Database::new_in_memory().await.unwrap();

    let id = db.create_tag("todelete").await.unwrap();

    // Usuń tag
    let deleted = db.delete_tag(id).await.unwrap();
    assert!(deleted);

    let tag = db.get_tag(id).await.unwrap();
    assert!(tag.is_none());

    // Spróbuj usunąć ponownie
    let deleted = db.delete_tag(id).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_delete_tag_by_name() {
    let db = Database::new_in_memory().await.unwrap();

    db.create_tag("todelete").await.unwrap();

    let deleted = db.delete_tag_by_name("todelete").await.unwrap();
    assert!(deleted);

    let tag = db.get_tag_by_name("todelete").await.unwrap();
    assert!(tag.is_none());

    // Spróbuj usunąć nieistniejący tag
    let deleted = db.delete_tag_by_name("nonexistent").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_tag_exists() {
    let db = Database::new_in_memory().await.unwrap();

    let id = db.create_tag("test").await.unwrap();

    assert!(db.tag_exists(id).await.unwrap());
    assert!(db.tag_exists_by_name("test").await.unwrap());
    assert!(!db.tag_exists(999).await.unwrap());
    assert!(!db.tag_exists_by_name("nonexistent").await.unwrap());
}

#[tokio::test]
async fn test_count_tags() {
    let db = Database::new_in_memory().await.unwrap();

    // Początkowo 0 tagów
    let count = db.count_tags().await.unwrap();
    assert_eq!(count, 0);

    // Dodaj 3 tagi
    let tag_names = vec!["tag1", "tag2", "tag3"];
    for name in &tag_names {
        db.create_tag(name).await.unwrap();
    }

    let count = db.count_tags().await.unwrap();
    assert_eq!(count, 3);
}

// ========== TESTOWANIE RELACJI PLIK-TAG ==========

#[tokio::test]
async fn test_add_and_remove_tag_from_file() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz plik i tag
    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    let tag_id = db.create_tag("important").await.unwrap();

    // Dodaj tag do pliku
    let added = db.add_tag_to_file(file_id, tag_id).await.unwrap();
    assert!(added);

    // Sprawdź czy plik ma tag
    let has_tag = db.file_has_tag(file_id, tag_id).await.unwrap();
    assert!(has_tag);

    // Pobierz tagi dla pliku
    let tags = db.get_tags_for_file(file_id).await.unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "important");

    // Usuń tag z pliku
    let removed = db.remove_tag_from_file(file_id, tag_id).await.unwrap();
    assert!(removed);

    let has_tag = db.file_has_tag(file_id, tag_id).await.unwrap();
    assert!(!has_tag);
}

#[tokio::test]
async fn test_add_tag_by_name_to_file() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    // Tag nie istnieje - zostanie utworzony automatycznie
    let added = db.add_tag_by_name_to_file(file_id, "work").await.unwrap();
    assert!(added);

    let tags = db.get_tags_for_file(file_id).await.unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "work");
}

#[tokio::test]
async fn test_add_duplicate_tag_to_file() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    let tag_id = db.create_tag("test").await.unwrap();

    // Dodaj tag pierwszy raz
    let added1 = db.add_tag_to_file(file_id, tag_id).await.unwrap();
    assert!(added1);

    // Spróbuj dodać ponownie - powinno zwrócić true (już istnieje)
    let added2 = db.add_tag_to_file(file_id, tag_id).await.unwrap();
    assert!(added2);

    // Nadal tylko jeden tag
    let tags = db.get_tags_for_file(file_id).await.unwrap();
    assert_eq!(tags.len(), 1);
}

#[tokio::test]
async fn test_remove_tag_by_name_from_file() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    db.add_tag_by_name_to_file(file_id, "work").await.unwrap();

    // Usuń tag po nazwie
    let removed = db
        .remove_tag_by_name_from_file(file_id, "work")
        .await
        .unwrap();
    assert!(removed);

    let tags = db.get_tags_for_file(file_id).await.unwrap();
    assert!(tags.is_empty());
}

#[tokio::test]
async fn test_get_files_with_tag() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let tag_id = db.create_tag("project").await.unwrap();

    // Utwórz 3 pliki z tagiem "project"
    for i in 0..3 {
        let file_id = db
            .create_file(
                PathBuf::from(format!("/project/file{}.txt", i)),
                100,
                format!("hash{}", i),
                lm,
                la,
                cr,
            )
            .await
            .unwrap();

        db.add_tag_to_file(file_id, tag_id).await.unwrap();
    }

    // Utwórz plik bez tagu
    db.create_file(
        PathBuf::from("/other/file.txt"),
        100,
        "hash".to_string(),
        lm,
        la,
        cr,
    )
    .await
    .unwrap();

    let files_with_tag = db.get_files_with_tag(tag_id).await.unwrap();
    assert_eq!(files_with_tag.len(), 3);
}

#[tokio::test]
async fn test_get_files_with_tag_name() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz pliki z różnymi tagami
    for i in 0..3 {
        let file_id = db
            .create_file(
                PathBuf::from(format!("/file{}.txt", i)),
                100,
                format!("hash{}", i),
                lm,
                la,
                cr,
            )
            .await
            .unwrap();

        if i < 2 {
            db.add_tag_by_name_to_file(file_id, "work").await.unwrap();
        } else {
            db.add_tag_by_name_to_file(file_id, "personal")
                .await
                .unwrap();
        }
    }

    let work_files = db.get_files_with_tag_name("work").await.unwrap();
    assert_eq!(work_files.len(), 2);

    let personal_files = db.get_files_with_tag_name("personal").await.unwrap();
    assert_eq!(personal_files.len(), 1);

    let nonexistent_files = db.get_files_with_tag_name("nonexistent").await.unwrap();
    assert!(nonexistent_files.is_empty());
}

#[tokio::test]
async fn test_get_tags_with_file_count() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz tagi
    let tags = vec!["work", "personal", "archive"];
    for tag_name in &tags {
        db.create_tag(tag_name).await.unwrap();
    }

    // Utwórz pliki i przypisz tagi
    for i in 0..5 {
        let file_id = db
            .create_file(
                PathBuf::from(format!("/file{}.txt", i)),
                100,
                format!("hash{}", i),
                lm,
                la,
                cr,
            )
            .await
            .unwrap();

        if i < 3 {
            // Pierwsze 3 pliki mają tag "work"
            db.add_tag_by_name_to_file(file_id, "work").await.unwrap();
        }
        if i < 2 {
            // Pierwsze 2 pliki mają tag "personal"
            db.add_tag_by_name_to_file(file_id, "personal")
                .await
                .unwrap();
        }
        // Tag "archive" nie jest przypisany do żadnego pliku
    }

    let tags_with_count = db.get_tags_with_file_count().await.unwrap();
    assert_eq!(tags_with_count.len(), 3);

    // Sprawdź liczbę plików dla każdego tagu
    for (tag, count) in tags_with_count {
        match tag.name.as_str() {
            "work" => assert_eq!(count, 3),
            "personal" => assert_eq!(count, 2),
            "archive" => assert_eq!(count, 0),
            _ => panic!("Nieoczekiwany tag"),
        }
    }
}

#[tokio::test]
async fn test_remove_all_tags_from_file() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    // Dodaj kilka tagów
    let tag_names = vec!["work", "important", "urgent"];
    for name in &tag_names {
        db.add_tag_by_name_to_file(file_id, name).await.unwrap();
    }

    let tags_before = db.get_tags_for_file(file_id).await.unwrap();
    assert_eq!(tags_before.len(), 3);

    // Usuń wszystkie tagi
    let removed_count = db.remove_all_tags_from_file(file_id).await.unwrap();
    assert_eq!(removed_count, 3);

    let tags_after = db.get_tags_for_file(file_id).await.unwrap();
    assert!(tags_after.is_empty());
}

#[tokio::test]
async fn test_cleanup_unused_tags() {
    let db = Database::new_in_memory().await.unwrap();

    // Utwórz kilka tagów
    let tag_names = vec!["used1", "used2", "unused1", "unused2"];
    for name in &tag_names {
        db.create_tag(name).await.unwrap();
    }

    // Utwórz plik i przypisz tylko 2 tagi
    let (lm, la, cr) = get_test_times();
    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    db.add_tag_by_name_to_file(file_id, "used1").await.unwrap();
    db.add_tag_by_name_to_file(file_id, "used2").await.unwrap();

    let tags_before = db.get_all_tags().await.unwrap();
    assert_eq!(tags_before.len(), 4);

    // Wyczyść nieużywane tagi
    let removed_count = db.cleanup_unused_tags().await.unwrap();
    assert_eq!(removed_count, 2); // unused1 i unused2

    let tags_after = db.get_all_tags().await.unwrap();
    assert_eq!(tags_after.len(), 2);

    // Sprawdź które tagi pozostały
    let remaining_names: Vec<String> = tags_after.iter().map(|t| t.name.clone()).collect();
    assert!(remaining_names.contains(&"used1".to_string()));
    assert!(remaining_names.contains(&"used2".to_string()));
}

#[tokio::test]
async fn test_add_tag_to_nonexistent_file() {
    let db = Database::new_in_memory().await.unwrap();

    let tag_id = db.create_tag("test").await.unwrap();

    // Spróbuj dodać tag do nieistniejącego pliku
    let added = db.add_tag_to_file(999, tag_id).await.unwrap();
    assert!(!added);
}

#[tokio::test]
async fn test_cascade_delete_when_tag_removed() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz plik i tag
    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    let tag_id = db.create_tag("test").await.unwrap();

    // Dodaj tag do pliku
    db.add_tag_to_file(file_id, tag_id).await.unwrap();

    // Usuń tag
    db.delete_tag(tag_id).await.unwrap();

    // Relacja w file_tags powinna zostać automatycznie usunięta (CASCADE)
    let has_tag = db.file_has_tag(file_id, tag_id).await.unwrap();
    assert!(!has_tag);
}

#[tokio::test]
async fn test_cascade_delete_when_file_removed() {
    let db = Database::new_in_memory().await.unwrap();
    let (lm, la, cr) = get_test_times();

    // Utwórz plik i tag
    let file_id = db
        .create_file(
            PathBuf::from("/test/file.txt"),
            1024,
            "hash".to_string(),
            lm,
            la,
            cr,
        )
        .await
        .unwrap();

    let tag_id = db.create_tag("test").await.unwrap();

    // Dodaj tag do pliku
    db.add_tag_to_file(file_id, tag_id).await.unwrap();

    // Usuń plik
    db.delete_file(file_id).await.unwrap();

    // Relacja w file_tags powinna zostać automatycznie usunięta (CASCADE)
    // Tag powinien nadal istnieć
    let tag_exists = db.tag_exists(tag_id).await.unwrap();
    assert!(tag_exists);
}
