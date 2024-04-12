use aw_core::encoding::latin1_to_string;

use crate::sqlite_wrap::SqliteValue;

#[derive(Debug)]
pub enum Row {
    MysqlRow(mysql::Row),
    SqliteRow {
        column_names: Vec<String>,
        row: Vec<SqliteValue>,
    },
}

impl Row {
    pub fn fetch_int(&self, name: &str) -> Option<i64> {
        match &self {
            Row::MysqlRow(row) => match row.get::<mysql::Value, _>(name) {
                Some(mysql::Value::Int(x)) => Some(x),
                _ => None,
            },
            Row::SqliteRow { column_names, row } => match column_names
                .iter()
                .enumerate()
                .find(|(_, cname)| *cname == name)
                .and_then(|(i, _)| row.get(i))
            {
                Some(SqliteValue::Int(x)) => Some(*x),
                _ => None,
            },
        }
    }

    pub fn fetch_string(&self, name: &str) -> Option<String> {
        match &self {
            Row::MysqlRow(row) => match row.get::<mysql::Value, _>(name) {
                Some(mysql::Value::Bytes(x)) => Some(latin1_to_string(&x)),
                _ => None,
            },
            Row::SqliteRow { column_names, row } => match column_names
                .iter()
                .enumerate()
                .find(|(_, cname)| *cname == name)
                .and_then(|(i, _)| row.get(i))
            {
                Some(SqliteValue::String(x)) => Some(x.clone()),
                _ => None,
            },
        }
    }
}
