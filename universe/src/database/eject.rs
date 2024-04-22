use aw_db::{aw_params, DatabaseResult, Row};

use super::UniverseDatabase;

pub trait EjectDB {
    fn init_eject(&self) -> DatabaseResult<()>;
    fn ejection_set(
        &self,
        address: u32,
        expiration: u32,
        creation: u32,
        comment: &str,
    ) -> DatabaseResult<()>;
    fn ejection_lookup(&self, address: u32) -> DatabaseResult<Option<EjectionQuery>>;
    fn ejection_next(&self, address: u32) -> DatabaseResult<Option<EjectionQuery>>;
    fn ejection_prev(&self, address: u32) -> DatabaseResult<Option<EjectionQuery>>;
    fn ejection_delete(&self, address: u32) -> DatabaseResult<()>;
    fn ejection_clean(&self, timestamp: u32) -> DatabaseResult<()>;
}

pub struct EjectionQuery {
    pub address: u32,
    pub expiration: u32,
    pub creation: u32,
    pub comment: String,
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

    fn ejection_set(
        &self,
        address: u32,
        expiration: u32,
        creation: u32,
        comment: &str,
    ) -> DatabaseResult<()> {
        // Check if ejection is already in the database
        let r = self.db.exec(
            r"SELECT * FROM awu_eject WHERE Address=?;",
            aw_params! {
                address
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let r = if rows.is_empty() {
            // Add the ejection if it is not already existent
            self.db.exec(
                r"INSERT INTO awu_eject (Expiration, Creation, Address, Comment)  
                VALUES(?, ?, ?, ?);",
                aw_params! {
                    expiration,
                    creation,
                    address,
                    comment
                },
            )
        } else {
            // Try to update the ejection if it is already present
            self.db.exec(
                r"UPDATE awu_eject SET Expiration=?, Creation=?, Comment=? 
                WHERE Address=?;",
                aw_params! {
                    expiration,
                    creation,
                    comment,
                    address
                },
            )
        };

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn ejection_lookup(&self, address: u32) -> DatabaseResult<Option<EjectionQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_eject WHERE Address=?;",
            aw_params! {
                address
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let row = match rows.first() {
            Some(row) => row,
            None => return DatabaseResult::Ok(None),
        };

        match fetch_ejection(row) {
            DatabaseResult::Ok(e) => DatabaseResult::Ok(Some(e)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn ejection_next(&self, address: u32) -> DatabaseResult<Option<EjectionQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_eject WHERE Address>? ORDER BY Address LIMIT 1;",
            aw_params! {
                address
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let row = match rows.first() {
            Some(row) => row,
            None => return DatabaseResult::Ok(None),
        };

        match fetch_ejection(row) {
            DatabaseResult::Ok(e) => DatabaseResult::Ok(Some(e)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn ejection_prev(&self, address: u32) -> DatabaseResult<Option<EjectionQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_eject WHERE Address<? ORDER BY Address DESC LIMIT 1;",
            aw_params! {
                address
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let row = match rows.first() {
            Some(row) => row,
            None => return DatabaseResult::Ok(None),
        };

        match fetch_ejection(row) {
            DatabaseResult::Ok(e) => DatabaseResult::Ok(Some(e)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn ejection_delete(&self, address: u32) -> DatabaseResult<()> {
        let r = self.db.exec(
            r"DELETE FROM awu_eject WHERE Address=?;",
            aw_params! {
                address
            },
        );

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn ejection_clean(&self, timestamp: u32) -> DatabaseResult<()> {
        let r = self.db.exec(
            r"DELETE FROM awu_eject WHERE Expiration>0 AND Expiration<?;",
            aw_params! {
                timestamp
            },
        );

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }
}

fn fetch_ejection(row: &Row) -> DatabaseResult<EjectionQuery> {
    let address = match row.fetch_int("Address").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let creation = match row.fetch_int("Creation").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let expiration = match row.fetch_int("Expiration").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let comment = match row.fetch_string("Comment") {
        Some(x) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    DatabaseResult::Ok(EjectionQuery {
        address,
        expiration,
        creation,
        comment,
    })
}
