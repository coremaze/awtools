//! RC4 implementation for Active Worlds 4 and 5
use rand::{thread_rng, RngCore};

use super::{AWCryptStream, StreamKeyError};

#[derive(Clone)]
pub struct AWCryptA4 {
    init_rand_buf: Vec<u8>,
    prga_index_a: usize,
    prga_index_b: usize,
    sbox: [u8; 256],
}

impl AWCryptA4 {
    /// Get the next XOR byte in the RC4 cipher.
    fn byte_a4(&mut self) -> u8 {
        self.prga_index_a = (self.prga_index_a + 1) % self.sbox.len();
        self.prga_index_b =
            (self.prga_index_b + self.sbox[self.prga_index_a] as usize) % self.sbox.len();
        self.sbox.swap(self.prga_index_a, self.prga_index_b);

        let result_index = (self.sbox[self.prga_index_a] as usize
            + self.sbox[self.prga_index_b] as usize)
            % self.sbox.len();

        self.sbox[result_index]
    }
}

impl AWCryptStream for AWCryptA4 {
    /// Create a new RC4 cipher with a new key.
    fn new() -> Self {
        // Create a new random sequence of bytes to use as the key.
        let mut new_random_key = [0u8; 256];
        thread_rng().fill_bytes(&mut new_random_key);

        // Use the new key to create a new instance of the cipher.
        Self::from_key(&new_random_key).expect(
            "The random key that was generated was not valid. This should be caught in tests.",
        )
    }

    /// Create an RC4 cipher using an existing key.
    fn from_key(key: &[u8]) -> Result<Self, StreamKeyError> {
        if key.len() < 16 {
            return Err(StreamKeyError::TooShort);
        }

        // Start sbox as [0..256; 256]
        let mut sbox: [u8; 256] = [0u8; 256];
        for (i, x) in sbox.iter_mut().enumerate() {
            *x = (i & u8::MAX as usize) as u8;
        }

        // ksa
        let mut j: usize = 0;
        for i in 0..sbox.len() {
            j = (j + sbox[i] as usize + key[i % key.len()] as usize) % sbox.len();
            sbox.swap(i, j);
        }

        Ok(Self {
            init_rand_buf: key.to_vec(),
            prga_index_a: 0,
            prga_index_b: 0,
            sbox,
        })
    }

    /// Get the initial key value used to set up the cipher
    fn get_initial_random_buffer(&self) -> Vec<u8> {
        self.init_rand_buf.clone()
    }

    /// Encrypt bytes, storing the result in the same buffer.
    fn encrypt_in_place(&mut self, buffer: &mut [u8]) {
        for x in buffer.iter_mut() {
            *x ^= self.byte_a4();
        }
    }

    /// Decrypt bytes, storing the result in the same buffer.
    fn decrypt_in_place(&mut self, buffer: &mut [u8]) {
        self.encrypt_in_place(buffer)
    }

    /// Encrypt bytes, returning a vector
    fn encrypt(&mut self, buffer: &[u8]) -> Vec<u8> {
        buffer
            .iter()
            .map(|x| x ^ self.byte_a4())
            .collect::<Vec<u8>>()
    }

    /// Decrypt bytes, returning a vector
    fn decrypt(&mut self, buffer: &[u8]) -> Vec<u8> {
        self.encrypt(buffer)
    }
}

impl Default for AWCryptA4 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_encrypt() {
        let mut a4_encrypt = AWCryptA4::new();
        let mut a4_decrypt = a4_encrypt.clone();
        let data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();
        let encrypted_data = a4_encrypt.encrypt(&data);
        let decrypted_data = a4_decrypt.decrypt(&encrypted_data);

        assert!(
            data == decrypted_data,
            "Encrypting and decrypting did not produce the original data."
        );
    }

    #[test]
    pub fn test_encrypt_in_place() {
        let mut a4_encrypt = AWCryptA4::new();
        let mut a4_inplace = a4_encrypt.clone();
        let mut data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();

        let encrypted_data = a4_encrypt.encrypt(&data);

        a4_inplace.encrypt_in_place(data.as_mut_slice());

        assert!(
            encrypted_data == data,
            "Results of encrypt and encrypt_in_place differ."
        );
    }
}
