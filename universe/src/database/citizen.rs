use super::Database;
use mysql::prelude::*;

pub trait CitizenDB {
    fn init_citizen(&self);
}

impl CitizenDB for Database {
    fn init_citizen(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_citizen ( 
            ID int(11) unsigned NOT NULL auto_increment, 
            Changed tinyint(1) NOT NULL default '0', 
            Name varchar(255) NOT NULL default '', 
            Password varchar(255) NOT NULL default '', 
            Email varchar(255) NOT NULL default '', 
            PrivPass varchar(255) NOT NULL default '', 
            Comment varchar(255) NOT NULL default '', 
            URL varchar(255) NOT NULL default '', 
            Immigration int(11) NOT NULL default '0', 
            Expiration int(11) NOT NULL default '0', 
            LastLogin int(11) NOT NULL default '0', 
            LastAddress int(11) NOT NULL default '0', 
            TotalTime int(11) NOT NULL default '0', 
            BotLimit int(11) NOT NULL default '0', 
            Beta tinyint(1) NOT NULL default '0', 
            CAVEnabled tinyint(1) NOT NULL default '0', 
            CAVTemplate int(11) NOT NULL default '0', 
            Enabled tinyint(1) NOT NULL default '1', 
            Privacy int(11) NOT NULL default '0', 
            Trial tinyint(1) NOT NULL default '0', 
            PRIMARY KEY  (ID), 
            UNIQUE KEY Index1 (Name), 
            KEY Index2 (Email) 
        ) 
        ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();

        // TODO: Add default administrator account if one does not exist yet.
    }
}
