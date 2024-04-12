//! AES implementation for Active Worlds 6
use rand::{thread_rng, RngCore};

use super::{AWCryptStream, StreamCipherError};

use aes::cipher::{KeyIvInit, StreamCipher};
type Aes256Ofb = ofb::Ofb<aes::Aes256>;

#[derive(Clone)]
pub struct AWCryptAES {
    init_rand_buf: Vec<u8>,
    cipher: Aes256Ofb,
}

impl AWCryptStream for AWCryptAES {
    /// Create a new AES cipher with a new key.
    fn new() -> Result<Self, StreamCipherError> {
        // Create a new random sequence of bytes to use as the key.
        let mut new_random_key = [0u8; 256];
        thread_rng().fill_bytes(&mut new_random_key);

        // Use the new key to create a new instance of the cipher.

        // This should never be invalid
        Self::from_key(&new_random_key)
    }

    /// Create an AES cipher using an existing key.
    fn from_key(src: &[u8]) -> Result<Self, StreamCipherError> {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];

        if src.len() < 0x20 {
            return Err(StreamCipherError::KeyTooShort);
        }

        iv.copy_from_slice(
            src.get(0x10..0x20)
                // Can't fail because we already reject keys that are too short
                .ok_or(StreamCipherError::ImplementationError)?,
        );

        for (i, j) in src.iter().rev().enumerate() {
            let key_index = i
                .checked_rem_euclid(key.len())
                // Can't fail because `key` is hardcoded to be 32 bytes
                .ok_or(StreamCipherError::ImplementationError)?;

            let key_value_ptr = key
                .get_mut(key_index)
                // Can't fail because key_index is constrainted to key.len()
                .ok_or(StreamCipherError::ImplementationError)?;

            *key_value_ptr = key_value_ptr.wrapping_add(*j);
        }

        let cipher = Aes256Ofb::new(&key.into(), &iv.into());

        Ok(Self {
            init_rand_buf: src.to_vec(),
            cipher,
        })
    }

    /// Get the initial key value used to set up the cipher
    fn get_initial_random_buffer(&self) -> Vec<u8> {
        self.init_rand_buf.clone()
    }

    /// Encrypt bytes, storing the result in the same buffer.
    fn encrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), StreamCipherError> {
        self.cipher
            .try_apply_keystream(buffer)
            .map_err(|_| StreamCipherError::ImplementationError)?;

        Ok(())
    }

    /// Decrypt bytes, storing the result in the same buffer.
    fn decrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), StreamCipherError> {
        self.encrypt_in_place(buffer)
    }

    /// Encrypt bytes, returning a vector
    fn encrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, StreamCipherError> {
        let mut encrypted = buffer.to_vec();
        self.encrypt_in_place(&mut encrypted)?;
        Ok(encrypted)
    }

    /// Decrypt bytes, returning a vector
    fn decrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, StreamCipherError> {
        let mut decrypted = buffer.to_vec();
        self.decrypt_in_place(&mut decrypted)?;
        Ok(decrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_encrypt() {
        let mut aes_encrypt = AWCryptAES::new().unwrap();
        let mut aes_decrypt = aes_encrypt.clone();
        let data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();
        let encrypted_data = aes_encrypt.encrypt(&data).unwrap();
        let decrypted_data = aes_decrypt.decrypt(&encrypted_data).unwrap();

        assert!(
            data == decrypted_data,
            "Encrypting and decrypting did not produce the original data."
        );
    }

    #[test]
    pub fn test_encrypt_in_place() {
        let mut aes_encrypt = AWCryptAES::new().unwrap();
        let mut aes_inplace = aes_encrypt.clone();
        let mut data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();

        let encrypted_data = aes_encrypt.encrypt(&data).unwrap();

        aes_inplace.encrypt_in_place(data.as_mut_slice()).unwrap();

        assert!(
            encrypted_data == data,
            "Results of encrypt and encrypt_in_place differ."
        );
    }
}
