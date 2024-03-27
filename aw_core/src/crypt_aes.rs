//! AES implementation for Active Worlds 6
use rand::{thread_rng, RngCore};

use super::{AWCryptStream, StreamKeyError};

use aes::cipher::{KeyIvInit, StreamCipher};
type Aes256Ofb = ofb::Ofb<aes::Aes256>;

#[derive(Clone)]
pub struct AWCryptAES {
    init_rand_buf: Vec<u8>,
    cipher: Aes256Ofb,
}

impl AWCryptStream for AWCryptAES {
    /// Create a new AES cipher with a new key.
    fn new() -> Self {
        // Create a new random sequence of bytes to use as the key.
        let mut new_random_key = [0u8; 256];
        thread_rng().fill_bytes(&mut new_random_key);

        // Use the new key to create a new instance of the cipher.
        Self::from_key(&new_random_key).expect(
            "The random key that was generated was not valid. This should be caught in tests.",
        )
    }

    /// Create an AES cipher using an existing key.
    fn from_key(src: &[u8]) -> Result<Self, StreamKeyError> {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];

        if src.len() < 0x20 {
            return Err(StreamKeyError::TooShort);
        }

        iv.copy_from_slice(&src[0x10..0x20]);

        for (i, j) in src.iter().rev().enumerate() {
            key[i % key.len()] = key[i % key.len()].wrapping_add(*j);
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
    fn encrypt_in_place(&mut self, buffer: &mut [u8]) {
        self.cipher.apply_keystream(buffer);
    }

    /// Decrypt bytes, storing the result in the same buffer.
    fn decrypt_in_place(&mut self, buffer: &mut [u8]) {
        self.encrypt_in_place(buffer)
    }

    /// Encrypt bytes, returning a vector
    fn encrypt(&mut self, buffer: &[u8]) -> Vec<u8> {
        let mut encrypted = buffer.to_vec();
        self.encrypt_in_place(&mut encrypted);
        encrypted
    }

    /// Decrypt bytes, returning a vector
    fn decrypt(&mut self, buffer: &[u8]) -> Vec<u8> {
        let mut decrypted = buffer.to_vec();
        self.decrypt_in_place(&mut decrypted);
        decrypted
    }
}

impl Default for AWCryptAES {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_encrypt() {
        let mut aes_encrypt = AWCryptAES::new();
        let mut aes_decrypt = aes_encrypt.clone();
        let data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();
        let encrypted_data = aes_encrypt.encrypt(&data);
        let decrypted_data = aes_decrypt.decrypt(&encrypted_data);

        assert!(
            data == decrypted_data,
            "Encrypting and decrypting did not produce the original data."
        );
    }

    #[test]
    pub fn test_encrypt_in_place() {
        let mut aes_encrypt = AWCryptAES::new();
        let mut aes_inplace = aes_encrypt.clone();
        let mut data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();

        let encrypted_data = aes_encrypt.encrypt(&data);

        aes_inplace.encrypt_in_place(data.as_mut_slice());

        assert!(
            encrypted_data == data,
            "Results of encrypt and encrypt_in_place differ."
        );
    }
}
