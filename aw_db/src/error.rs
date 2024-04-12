#[derive(thiserror::Error, Debug)]
pub enum DatabaseOpenError {
    #[error("Couldn't create the MySQL conneciton pool: {0}")]
    ConnectionPoolFailure(#[from] mysql::Error),
    #[error("Couldn't open the Sqlite database: {0}")]
    SqliteOpenFailure(#[from] rusqlite::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum DatabaseExecError {
    #[error("Encountered mysql error {0}")]
    MysqlError(#[from] mysql::Error),
    #[error("Encountered sqlite error {0}")]
    SqliteError(#[from] rusqlite::Error),
}

#[derive(Debug, Copy, Clone)]
pub enum DatabaseResult<T> {
    Ok(T),
    DatabaseError,
}

impl<T> DatabaseResult<T> {
    pub fn is_err(&self) -> bool {
        match self {
            DatabaseResult::Ok(_) => false,
            DatabaseResult::DatabaseError => true,
        }
    }
}
