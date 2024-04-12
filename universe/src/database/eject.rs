use aw_db::DatabaseResult;

use super::UniverseDatabase;

pub trait EjectDB {
    fn init_eject(&self) -> DatabaseResult<()>;
}

impl EjectDB for UniverseDatabase {
    fn init_eject(&self) -> DatabaseResult<()> {
        let unsigned = self.db.unsigned_str();
        let auto_increment_not_null = self.db.auto_increment_not_null();
        let r = self.db.exec(
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
