use super::Database;
use mysql::prelude::*;

pub trait LicenseDB {
    fn init_license(&self);
}

impl LicenseDB for Database {
    fn init_license(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        // "Range" has been changed to "WorldSize" because range is now a keyword.
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_license ( 
                ID int(11) NOT NULL auto_increment, 
                Name varchar(50) NOT NULL default '', 
                Password varchar(255) NOT NULL default '', 
                Email varchar(255) NOT NULL default '', 
                Comment varchar(255) NOT NULL default '', 
                Creation int(11) NOT NULL default '0', 
                Expiration int(11) NOT NULL default '0', 
                LastStart int(11) NOT NULL default '0', 
                LastAddress int(11) NOT NULL default '0', 
                Users int(11) NOT NULL default '0', 
                WorldSize int(11) NOT NULL default '0', 
                Hidden tinyint(1) NOT NULL default '0', 
                Changed tinyint(1) NOT NULL default '0', 
                Tourists tinyint(1) NOT NULL default '0', 
                Voip tinyint(1) NOT NULL default '0', 
                Plugins tinyint(1) NOT NULL default '0', 
                PRIMARY KEY  (ID), UNIQUE KEY Index1 (Name) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }
}
