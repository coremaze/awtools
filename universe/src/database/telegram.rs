use super::Database;
use crate::database;
use aw_core::ReasonCode;
use mysql::prelude::*;
use mysql::*;

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
    fn telegram_mark_delivered(&self, telegram_id: u32) -> Result<(), ReasonCode>;
}

impl TelegramDB for Database {
    fn init_telegram(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_telegram ( 
                ID int(11) NOT NULL auto_increment, 
                Citizen int(11) unsigned NOT NULL default '0', 
                `From` int(11) unsigned NOT NULL default '0', 
                `Timestamp` int(11) unsigned NOT NULL default '0', 
                Message text NOT NULL, 
                Delivered tinyint(1) NOT NULL default '0', 
                PRIMARY KEY  (ID), 
                KEY Index1 (Citizen) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }

    fn telegram_add(
        &self,
        to: u32,
        from: u32,
        timestamp: u32,
        message: &str,
    ) -> Result<(), ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        conn.exec_drop(
            r"INSERT INTO awu_telegram (Citizen,`From`,Timestamp,Message,Delivered) 
            VALUES(:to, :from, :timestamp, :message, 0)",
            params! {
                "to" => to,
                "from" => from,
                "timestamp" => timestamp,
                "message" => &message,
            },
        )
        .map_err(|_| ReasonCode::DatabaseError)?;

        Ok(())
    }

    fn telegram_get_undelivered(&self, citizen_id: u32) -> Vec<TelegramQuery> {
        let mut telegrams = Vec::<TelegramQuery>::new();
        let mut conn = match self.conn() {
            Ok(x) => x,
            Err(_) => return telegrams,
        };

        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_telegram WHERE Citizen=:id AND Delivered=0 
                ORDER BY Timestamp",
                params! {
                    "id" => citizen_id,
                },
            )
            .unwrap_or(Vec::<Row>::new());

        for row in &rows {
            if let Ok(telegram) = fetch_telegram(row) {
                telegrams.push(telegram);
            }
        }

        telegrams
    }

    fn telegram_get_all(&self, citizen_id: u32) -> Vec<TelegramQuery> {
        let mut telegrams = Vec::<TelegramQuery>::new();
        let mut conn = match self.conn() {
            Ok(x) => x,
            Err(_) => return telegrams,
        };

        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_telegram WHERE Citizen=:id  
                ORDER BY Timestamp",
                params! {
                    "id" => citizen_id,
                },
            )
            .unwrap_or(Vec::<Row>::new());

        for row in &rows {
            if let Ok(telegram) = fetch_telegram(row) {
                telegrams.push(telegram);
            }
        }

        telegrams
    }

    fn telegram_mark_delivered(&self, telegram_id: u32) -> Result<(), ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        conn.exec_drop(
            r"UPDATE awu_telegram SET Delivered=1 
            WHERE ID=:telegram_id;",
            params! {
                "telegram_id" => telegram_id,
            },
        )
        .map_err(|x| ReasonCode::DatabaseError)?;

        Ok(())
    }
}

fn fetch_telegram(row: &Row) -> Result<TelegramQuery, ReasonCode> {
    let id: u32 = database::fetch_int(row, "ID")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let citizen: u32 = database::fetch_int(row, "Citizen")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let from: u32 = database::fetch_int(row, "From")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let timestamp: u32 = database::fetch_int(row, "Timestamp")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let message: String =
        database::fetch_string(row, "Message").ok_or(ReasonCode::DatabaseError)?;

    let delivered: u32 = database::fetch_int(row, "Delivered")
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
