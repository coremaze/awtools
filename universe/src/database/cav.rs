use aw_db::DatabaseResult;

use super::UniverseDatabase;

pub trait CavDB {
    fn init_cav(&self) -> DatabaseResult<()>;
}

impl CavDB for UniverseDatabase {
    fn init_cav(&self) -> DatabaseResult<()> {
        let auto_increment_not_null = self.db.auto_increment_not_null();
        let unsigned = self.db.unsigned_str();

        let r = self.db.exec(
            format!(
                r"CREATE TABLE IF NOT EXISTS awu_cav ( 
            Citizen INTEGER {unsigned} NOT NULL default '0', 
            Template INTEGER NOT NULL default '0', 
            Changed tinyint(4) NOT NULL default '0', 
            Keyframe1Scale float NOT NULL default '0', 
            Keyframe2Scale float NOT NULL default '0', 
            Height float NOT NULL default '0', 
            SkinColor INTEGER NOT NULL default '0', 
            HairColor INTEGER NOT NULL default '0',
            PRIMARY KEY (Citizen, Template)
        );"
            ),
            vec![],
        );

        if r.is_err() {
            return DatabaseResult::DatabaseError;
        }

        let r = self.db.exec(
            format!(
                r"CREATE TABLE IF NOT EXISTS awu_cav_template ( 
                ID INTEGER PRIMARY KEY {auto_increment_not_null}, 
                Changed tinyint(4) NOT NULL default '0', 
                Type INTEGER NOT NULL default '0', 
                Rating INTEGER NOT NULL default '0', 
                Name varchar(255) default '', 
                Model varchar(255) NOT NULL default ''
            );"
            ),
            vec![],
        );

        if r.is_err() {
            return DatabaseResult::DatabaseError;
        }

        DatabaseResult::Ok(())
    }
}
