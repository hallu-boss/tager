use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Błąd SQL: {0}")]
    Sql(#[from] sqlx::Error),
    
    #[error("Błąd I/O: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Błąd operacji na bazie danych: {0}")]
    OperationFailed(String),
}