use super::Database;
use mysql::prelude::*;

pub trait ContactDB {
    fn init_contact(&self);
}

impl ContactDB for Database {
    fn init_contact(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_contact ( 
                Citizen int(11) unsigned NOT NULL default '0', 
                Contact int(11) unsigned NOT NULL default '0', 
                Options int(11) unsigned NOT NULL default '0', 
                Changed tinyint(1) NOT NULL default '0', 
                PRIMARY KEY  (Citizen,Contact), 
                KEY Index1 (Contact,Citizen) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }
}
