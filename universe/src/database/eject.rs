use super::{Database, DatabaseResult};

pub trait EjectDB {
    fn init_eject(&self) -> DatabaseResult<()>;
}

impl EjectDB for Database {
    fn init_eject(&self) -> DatabaseResult<()> {
        let unsigned = self.unsigned_str();
        let auto_increment_not_null = self.auto_increment_not_null();
        let r = self.exec(
            format!(
                r"CREATE TABLE IF NOT EXISTS awu_eject ( 
                ID INTEGER PRIMARY KEY {auto_increment_not_null}, 
                Expiration INTEGER NOT NULL default '0', 
                Creation INTEGER NOT NULL default '0', 
                Address INTEGER {unsigned} NOT NULL default '0', 
                Comment varchar(255) NOT NULL default '', 
                Changed tinyint(1) NOT NULL default '0'
            );"
            ),
            vec![],
        );

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }
}
