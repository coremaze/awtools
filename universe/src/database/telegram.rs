use super::Database;
use mysql::prelude::*;

pub trait TelegramDB {
    fn init_telegram(&self);
}

impl TelegramDB for Database {
    fn init_telegram(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        // "Range" has been changed to "WorldSize" because range is now a keyword.
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_telegram ( 
                ID int(11) NOT NULL auto_increment, 
                Citizen int(11) unsigned NOT NULL default '0', 
                `From` int(11) unsigned NOT NULL default '0', 
                `Timestamp` int(11) unsigned NOT NULL default '0', 
                Message text NOT NULL, 
                PRIMARY KEY  (ID), 
                KEY Index1 (Citizen) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }
}
