use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(String),
    #[error("not found: {entity} #{id}")]
    NotFound { entity: String, id: String },
    #[error("migration error: {0}")]
    Migration(String),
    #[error("constraint violation: {0}")]
    Constraint(String),
}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        StorageError::Database(e.to_string())
    }
}
