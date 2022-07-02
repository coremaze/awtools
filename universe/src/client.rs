use crate::{AWConnection, AWCryptRSA};

pub struct Client {
    pub connection: AWConnection,
    pub dead: bool,
    pub rsa: AWCryptRSA,
}

impl Client {
    pub fn new(connection: AWConnection) -> Self {
        Self {
            connection,
            dead: false,
            rsa: AWCryptRSA::new(),
        }
    }
}
