use super::{AWRow, Database, DatabaseResult};
use crate::aw_params;

#[derive(Debug, Clone)]
pub struct TelegramQuery {
    pub id: u32,
    pub citizen: u32,
    pub from: u32,
    pub timestamp: u32,
    pub message: String,
    pub delivered: u32,
}

pub trait TelegramDB {
    fn init_telegram(&self) -> DatabaseResult<()>;
    fn telegram_add(&self, to: u32, from: u32, timestamp: u32, message: &str)
        -> DatabaseResult<()>;
    fn telegram_get_undelivered(&self, citizen_id: u32) -> DatabaseResult<Vec<TelegramQuery>>;
    fn telegram_get_all(&self, citizen_id: u32) -> DatabaseResult<Vec<TelegramQuery>>;
    fn telegram_mark_delivered(&self, telegram_id: u32) -> DatabaseResult<()>;
}

impl TelegramDB for Database {
    fn init_telegram(&self) -> DatabaseResult<()> {
        let auto_increment_not_null = self.auto_increment_not_null();
        let unsigned = self.unsigned_str();
        let statement = format!(
            r"CREATE TABLE IF NOT EXISTS awu_telegram ( 
            ID INTEGER PRIMARY KEY {auto_increment_not_null}, 
            Citizen INTEGER {unsigned} NOT NULL default '0', 
            `From` INTEGER {unsigned} NOT NULL default '0', 
            `Timestamp` INTEGER {unsigned} NOT NULL default '0', 
            Message text NOT NULL, 
            Delivered tinyint(1) NOT NULL default '0'
        );"
        );

        let r = self.exec(statement, vec![]);

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn telegram_add(
        &self,
        to: u32,
        from: u32,
        timestamp: u32,
        message: &str,
    ) -> DatabaseResult<()> {
        let r = self.exec(
            r"INSERT INTO awu_telegram (Citizen,`From`,Timestamp,Message,Delivered) 
            VALUES(?, ?, ?, ?, 0)",
            aw_params! {
                to,
                from,
                timestamp,
                &message
            },
        );

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn telegram_get_undelivered(&self, citizen_id: u32) -> DatabaseResult<Vec<TelegramQuery>> {
        let r = self.exec(
            r"SELECT * FROM awu_telegram WHERE Citizen=? AND Delivered=0 
            ORDER BY Timestamp",
            aw_params! {
                citizen_id
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let mut telegrams = Vec::<TelegramQuery>::new();
        for row in &rows {
            match fetch_telegram(row) {
                DatabaseResult::Ok(telegram) => telegrams.push(telegram),
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            }
        }

        DatabaseResult::Ok(telegrams)
    }

    fn telegram_get_all(&self, citizen_id: u32) -> DatabaseResult<Vec<TelegramQuery>> {
        let r = self.exec(
            r"SELECT * FROM awu_telegram WHERE Citizen=?  
                ORDER BY Timestamp",
            aw_params! {
               citizen_id
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let mut telegrams = Vec::<TelegramQuery>::new();
        for row in &rows {
            match fetch_telegram(row) {
                DatabaseResult::Ok(telegram) => telegrams.push(telegram),
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            }
        }

        DatabaseResult::Ok(telegrams)
    }

    fn telegram_mark_delivered(&self, telegram_id: u32) -> DatabaseResult<()> {
        let r = self.exec(
            r"UPDATE awu_telegram SET Delivered=1 
            WHERE ID=?;",
            aw_params! {
                telegram_id
            },
        );

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }
}

fn fetch_telegram(row: &AWRow) -> DatabaseResult<TelegramQuery> {
    let id = match row.fetch_int("ID").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let citizen = match row.fetch_int("Citizen").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let from = match row.fetch_int("From").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let timestamp = match row.fetch_int("Timestamp").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let message = match row.fetch_string("Message") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let delivered = match row.fetch_int("Delivered").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    DatabaseResult::Ok(TelegramQuery {
        id,
        citizen,
        from,
        timestamp,
        message,
        delivered,
    })
}
