use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Błąd SQL: {0}")]
    Sql(#[from] sqlx::Error),
}

pub enum FilesOrderBy {
    Id,
    Path,
}

impl FilesOrderBy {
    pub fn to_sql_clause(&self) -> &'static str {
        match self {
            FilesOrderBy::Id => "ORDER BY f.id",
            FilesOrderBy::Path => "ORDER BY f.path",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileWithTags {
    pub id: i64,
    pub path: String,
    pub tags: Vec<String>,
}