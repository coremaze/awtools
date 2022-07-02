use rand::{thread_rng, RngCore};
use rsaref_rs::*;

pub struct AWCryptRSA {
    random_struct: RandomStruct,
    public_key: Option<RSAPublicKey>,
    private_key: Option<RSAPrivateKey>,
}

pub enum RSAKey {
    Public,
    Private,
}

impl AWCryptRSA {
    pub fn new() -> Self {
        let random_struct = RandomStruct::new();

        let proto_key = RSAProtoKey {
            bits: 512,
            use_fermat4: true,
        };

        let (pub_key, priv_key) = generate_pem_keys(&proto_key)
            .expect("Failed to generate RSA keys");

        Self {
            random_struct,
            public_key: Some(pub_key),
            private_key: Some(priv_key),
        }
    }

    pub fn set_private_key(&mut self, private_key: RSAPrivateKey) {
        self.public_key = Some(private_key.public_key());
        self.private_key = Some(private_key);
    }

    pub fn decode_private_key(&mut self, data: &[u8]) -> Result<(), RSAError> {
        let private_key = RSAPrivateKey::decode(data).map_err(|_| RSAError::Key)?;
        self.set_private_key(private_key);
        Ok(())
    }

    pub fn encode_private_key(&self) -> Option<Vec<u8>> {
        if let Some(key) = &self.private_key {
            Some(key.encode())
        }
        else {
            None
        }
    }

    pub fn set_public_key(&mut self, public_key: RSAPublicKey) {
        self.public_key = Some(public_key);
    }

    pub fn encode_public_key(&self) -> Option<Vec<u8>> {
        if let Some(key) = &self.public_key {
            Some(key.encode())
        }
        else {
            None
        }
    }

    pub fn decode_public_key(&mut self, data: &[u8]) -> Result<(), RSAError> {
        let public_key = RSAPublicKey::decode(data).map_err(|_| RSAError::Key)?;
        self.set_public_key(public_key);
        Ok(())
    }

    pub fn randomize(&mut self) {
        let mut random_buffer = [0u8; 256];
        loop {
            if self.random_struct.get_random_bytes_needed() == 0 {
                break;
            }
            thread_rng().fill_bytes(&mut random_buffer);
            self.random_struct.random_update(&random_buffer);
        }
        self.random_struct.get_random_bytes_needed();
    }

    pub fn decrypt_private(&self, src: &[u8]) -> Result<Vec<u8>, RSAError> {
        let private_key = self.private_key.as_ref().ok_or(RSAError::Key)?;
        private_key.decrypt(src)
    }

    pub fn decrypt_public(&self, src: &[u8]) -> Result<Vec<u8>, RSAError> {
        let public_key = self.public_key.as_ref().ok_or(RSAError::Key)?;
        public_key.decrypt(src)
    }

    pub fn encrypt_private(&self, src: &[u8]) -> Result<Vec<u8>, RSAError> {
        let private_key = self.private_key.as_ref().ok_or(RSAError::Key)?;
        private_key.encrypt(src)
    }

    pub fn encrypt_public(&mut self, src: &[u8]) -> Result<Vec<u8>, RSAError> {
        let public_key = self.public_key.as_ref().ok_or(RSAError::Key)?;
        public_key.encrypt(src, &mut self.random_struct)
    }

    pub fn encrypt(&mut self, src: &[u8], key_type: RSAKey) -> Result<Vec<u8>, RSAError> {
        match key_type {
            RSAKey::Public => self.encrypt_public(src),
            RSAKey::Private => self.encrypt_private(src),
        }
    }

    pub fn decrypt(&self, src: &[u8], key_type: RSAKey) -> Result<Vec<u8>, RSAError> {
        match key_type {
            RSAKey::Public => self.decrypt_public(src),
            RSAKey::Private => self.decrypt_private(src),
        }
    }
}

impl Default for AWCryptRSA {
    fn default() -> Self {
        let random_struct = RandomStruct::new();
        Self {
            random_struct,
            public_key: None,
            private_key: None,
        }
    }
}
