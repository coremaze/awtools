use super::{AWRow, Database};
use crate::aw_params;
use aw_core::ReasonCode;

type Result<T, E> = std::result::Result<T, E>;

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
    fn init_telegram(&self);
    fn telegram_add(
        &self,
        to: u32,
        from: u32,
        timestamp: u32,
        message: &str,
    ) -> Result<(), ReasonCode>;
    fn telegram_get_undelivered(&self, citizen_id: u32) -> Vec<TelegramQuery>;
    fn telegram_get_all(&self, citizen_id: u32) -> Vec<TelegramQuery>;
    fn telegram_mark_delivered(&self, telegram_id: u32);
}

impl TelegramDB for Database {
    fn init_telegram(&self) {
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

        self.exec(statement, vec![]);
    }

    fn telegram_add(
        &self,
        to: u32,
        from: u32,
        timestamp: u32,
        message: &str,
    ) -> Result<(), ReasonCode> {
        self.exec(
            r"INSERT INTO awu_telegram (Citizen,`From`,Timestamp,Message,Delivered) 
            VALUES(?, ?, ?, ?, 0)",
            aw_params! {
                to,
                from,
                timestamp,
                &message
            },
        );

        Ok(())
    }

    fn telegram_get_undelivered(&self, citizen_id: u32) -> Vec<TelegramQuery> {
        let mut telegrams = Vec::<TelegramQuery>::new();

        let rows = self.exec(
            r"SELECT * FROM awu_telegram WHERE Citizen=? AND Delivered=0 
                ORDER BY Timestamp",
            aw_params! {
                citizen_id
            },
        );

        for row in &rows {
            if let Ok(telegram) = fetch_telegram(row) {
                telegrams.push(telegram);
            }
        }

        telegrams
    }

    fn telegram_get_all(&self, citizen_id: u32) -> Vec<TelegramQuery> {
        let mut telegrams = Vec::<TelegramQuery>::new();

        let rows = self.exec(
            r"SELECT * FROM awu_telegram WHERE Citizen=?  
                ORDER BY Timestamp",
            aw_params! {
               citizen_id
            },
        );

        for row in &rows {
            if let Ok(telegram) = fetch_telegram(row) {
                telegrams.push(telegram);
            }
        }

        telegrams
    }

    fn telegram_mark_delivered(&self, telegram_id: u32) {
        self.exec(
            r"UPDATE awu_telegram SET Delivered=1 
            WHERE ID=?;",
            aw_params! {
                telegram_id
            },
        );
    }
}

fn fetch_telegram(row: &AWRow) -> Result<TelegramQuery, ReasonCode> {
    let id: u32 = row
        .fetch_int("ID")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let citizen: u32 = row
        .fetch_int("Citizen")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let from: u32 = row
        .fetch_int("From")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let timestamp: u32 = row
        .fetch_int("Timestamp")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let message: String = row
        .fetch_string("Message")
        .ok_or(ReasonCode::DatabaseError)?;

    let delivered: u32 = row
        .fetch_int("Delivered")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    Ok(TelegramQuery {
        id,
        citizen,
        from,
        timestamp,
        message,
        delivered,
    })
}
