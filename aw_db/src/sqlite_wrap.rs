use rusqlite::params_from_iter;

use crate::{error::DatabaseExecError, Row};

#[derive(Debug)]
pub enum SqliteValue {
    None,
    Int(i64),
    String(String),
}

pub(crate) fn sqlite_exec(
    conn: &rusqlite::Connection,
    statement: impl AsRef<str>,
    parameters: Vec<String>,
) -> Result<Vec<Row>, DatabaseExecError> {
    let mut stmt = conn.prepare(statement.as_ref())?;

    let column_names = stmt
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    let mut rows = stmt.query(params_from_iter(parameters.iter()))?;

    let mut rows_vec = Vec::<Row>::new();
    while let Ok(Some(row)) = rows.next() {
        let mut row_vec = Vec::<SqliteValue>::new();
        for i in 0..column_names.len() {
            if let Ok(num) = row.get::<usize, i64>(i) {
                row_vec.push(SqliteValue::Int(num))
            } else if let Ok(b) = row.get::<usize, String>(i) {
                row_vec.push(SqliteValue::String(b))
            } else {
                row_vec.push(SqliteValue::None)
            }
        }
        rows_vec.push(Row::SqliteRow {
            column_names: column_names.clone(),
            row: row_vec,
        });
    }
    Ok(rows_vec)
}
