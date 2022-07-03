use super::Database;
use mysql::prelude::*;

pub trait AttribDB {
    fn init_attrib(&self);
}

impl AttribDB for Database {
    fn init_attrib(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_attrib ( 
            ID int(11) NOT NULL default '0', 
            Changed tinyint(1) NOT NULL default '0', 
            Value varchar(255) NOT NULL default '', 
            PRIMARY KEY  (ID) 
        ) ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();

        // TODO: There are some default values to fill in here which are not important yet
    }
}
