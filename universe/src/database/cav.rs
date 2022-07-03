use super::Database;
use mysql::prelude::*;

pub trait CavDB {
    fn init_cav(&self);
}

impl CavDB for Database {
    fn init_cav(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_cav ( 
                Citizen int(11) unsigned NOT NULL default '0', 
                Template int(11) NOT NULL default '0', 
                Changed tinyint(4) NOT NULL default '0', 
                Keyframe1Scale float NOT NULL default '0', 
                Keyframe2Scale float NOT NULL default '0', 
                Height float NOT NULL default '0', 
                SkinColor int(11) NOT NULL default '0', 
                HairColor int(11) NOT NULL default '0', 
                PRIMARY KEY  (Citizen,Template) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_cav_template ( 
                ID int(11) NOT NULL auto_increment, 
                Changed tinyint(4) NOT NULL default '0', 
                Type int(11) NOT NULL default '0', 
                Rating int(11) NOT NULL default '0', 
                Name varchar(255) default '', 
                Model varchar(255) NOT NULL default '', 
                PRIMARY KEY  (ID) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }
}
