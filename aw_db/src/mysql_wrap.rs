use mysql::prelude::Queryable;

use crate::{error::DatabaseExecError, Row};

pub(crate) fn mysql_exec(
    pool: &mysql::Pool,
    statement: impl AsRef<str>,
    parameters: Vec<String>,
) -> Result<Vec<Row>, DatabaseExecError> {
    let mut conn = pool.get_conn()?;
    let r: mysql::Result<Vec<mysql::Row>> = conn.exec(statement.as_ref(), parameters.clone());

    Ok(r?
        .iter()
        .map(|r| Row::MysqlRow(r.clone()))
        .collect::<Vec<Row>>())
}
