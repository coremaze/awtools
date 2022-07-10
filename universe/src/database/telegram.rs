use super::Database;
use aw_core::ReasonCode;
use mysql::prelude::*;
use mysql::*;

type Result<T, E> = std::result::Result<T, E>;

pub struct TelegramQuery {
    id: u32,
    citizen: u32,
    from: u32,
    timestamp: u32,
    message: String,
    delivered: u32,
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
}
