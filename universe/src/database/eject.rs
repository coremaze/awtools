use super::Database;
use mysql::prelude::*;

pub trait EjectDB {
    fn init_eject(&self);
}

impl EjectDB for Database {
    fn init_eject(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_eject ( 
                ID int(11) NOT NULL auto_increment, 
                Expiration int(11) NOT NULL default '0', 
                Creation int(11) NOT NULL default '0', 
                Address int(11) unsigned NOT NULL default '0', 
                Comment varchar(255) NOT NULL default '', 
                Changed tinyint(1) NOT NULL default '0', 
                PRIMARY KEY  (ID) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }
}
