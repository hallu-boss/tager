mod error;
mod types;
mod database;

pub use error::DbError;
pub use types::{FilesOrderBy, FileWithTags};
pub use database::Database;

mod tests;