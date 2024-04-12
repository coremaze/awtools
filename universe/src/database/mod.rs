use aw_db::{Database, DatabaseConfig, DatabaseOpenError};

use crate::configuration::UniverseConfig;

pub use self::attrib::AttribDB;
pub use self::cav::CavDB;
pub use self::citizen::CitizenDB;
pub use self::contact::ContactDB;
pub use self::eject::EjectDB;
pub use self::license::LicenseDB;
pub use self::telegram::TelegramDB;
pub mod attrib;
pub mod cav;
pub mod citizen;
pub mod contact;
pub mod eject;
pub mod license;
pub mod telegram;

pub struct UniverseDatabase {
    db: aw_db::Database,
}

impl UniverseDatabase {
    pub fn new(
        config: DatabaseConfig,
        universe_config: &UniverseConfig,
    ) -> Result<Self, DatabaseOpenError> {
        let db = Database::new(config)?;
        let unidb = UniverseDatabase { db };

        unidb.init_tables(universe_config);

        Ok(unidb)
    }

    fn init_tables(&self, universe_config: &UniverseConfig) {
        self.init_attrib(universe_config);
        self.init_citizen();
        self.init_contact();
        self.init_license();
        self.init_telegram();
        self.init_cav();
        self.init_eject();
    }
}
