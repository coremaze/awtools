//! RC4 implementation for Active Worlds 4 and 5
use rand::{thread_rng, RngCore};

use super::{AWCryptStream, StreamCipherError};

#[derive(Debug, Clone)]
pub struct AWCryptA4 {
    init_rand_buf: Vec<u8>,
    prga_index_a: usize,
    prga_index_b: usize,
    sbox: [u8; 256],
}

impl AWCryptA4 {
    /// Get the next XOR byte in the RC4 cipher.
    fn byte_a4(&mut self) -> Result<u8, StreamCipherError> {
        self.increment_prga_index_a()?;
        self.update_prga_index_b()?;
        self.swap_sbox_values();
        self.get_result_data()
    }

    fn increment_prga_index_a(&mut self) -> Result<(), StreamCipherError> {
        // prga_index_a increments by 1, or wraps around the length of sbox
        self.prga_index_a = self
            .prga_index_a
            .checked_add(1)
            // Can't fail since usize is bigger than the length of sbox
            .ok_or(StreamCipherError::ImplementationError)?
            .checked_rem_euclid(self.sbox.len())
            // Can't fail because sbox.len() is always 256, not 0
            .ok_or(StreamCipherError::ImplementationError)?;
        Ok(())
    }

    fn update_prga_index_b(&mut self) -> Result<(), StreamCipherError> {
        self.prga_index_b = self
            .prga_index_b
            .checked_add(self.get_sbox_a_index()?)
            // Can't fail because both values are derived from u8s, so the combined max is 510
            .ok_or(StreamCipherError::ImplementationError)?
            // Constrain the result to be an index inside sbox, 0..256
            .checked_rem_euclid(self.sbox.len())
            // Can't fail because sbox.len() is always 256
            .ok_or(StreamCipherError::ImplementationError)?;

        Ok(())
    }

    fn swap_sbox_values(&mut self) {
        self.sbox.swap(self.prga_index_a, self.prga_index_b);
    }

    fn get_sbox_a_index(&self) -> Result<usize, StreamCipherError> {
        Ok(usize::from(
            *self
                .sbox
                // sbox[prga_index_a]
                .get(self.prga_index_a)
                // Can't fail because prga_index_a wraps at sbox.len(), which is always 256
                .ok_or(StreamCipherError::ImplementationError)?,
        ))
    }

    fn get_sbox_b_index(&self) -> Result<usize, StreamCipherError> {
        Ok(usize::from(
            *self
                .sbox
                // sbox[prga_index_b]
                .get(self.prga_index_b)
                // Can't fail because prga_index_b wraps at sbox.len(), which is always 256
                .ok_or(StreamCipherError::ImplementationError)?,
        ))
    }

    fn get_result_index(&self) -> Result<usize, StreamCipherError> {
        self.get_sbox_a_index()?
            .checked_add(self.get_sbox_b_index()?)
            // Can't fail because the max of this operation is 510
            .ok_or(StreamCipherError::ImplementationError)?
            .checked_rem_euclid(self.sbox.len())
            // Can't fail because sbox.len() is 256
            .ok_or(StreamCipherError::ImplementationError)
    }

    fn get_result_data(&self) -> Result<u8, StreamCipherError> {
        let index = self.get_result_index()?;

        Ok(*self
            .sbox
            .get(index)
            // Can't fail because the index is constrained to 0..sbox.len()
            .ok_or(StreamCipherError::ImplementationError)?)
    }
}

impl AWCryptStream for AWCryptA4 {
    /// Create a new RC4 cipher with a new key.
    fn new() -> Result<Self, StreamCipherError> {
        // Create a new random sequence of bytes to use as the key.
        let mut new_random_key = [0u8; 256];
        thread_rng().fill_bytes(&mut new_random_key);

        // Use the new key to create a new instance of the cipher.
        // This should never be invalid
        Self::from_key(&new_random_key)
    }

    /// Create an RC4 cipher using an existing key.
    fn from_key(key: &[u8]) -> Result<Self, StreamCipherError> {
        if key.len() < 16 {
            return Err(StreamCipherError::KeyTooShort);
        }

        // Start sbox as [0..256; 256]
        let mut sbox: [u8; 256] = [0u8; 256];
        for (i, x) in sbox.iter_mut().enumerate() {
            *x = i
                .try_into()
                // Since i will only ever get up to 255, this will never happen
                .map_err(|_| StreamCipherError::ImplementationError)?;
        }

        // ksa
        let mut j: usize = 0;
        for i in 0..sbox.len() {
            // ok_or will never happen since key.len() must be greater than 15
            let key_index = i
                .checked_rem_euclid(key.len())
                .ok_or(StreamCipherError::ImplementationError)?;
            // ok_or will never happen since key_index was just constrained to key.len()
            let key_value = *key
                .get(key_index)
                .ok_or(StreamCipherError::ImplementationError)?;
            // ok_or will never happen since i goes between 0 and sbox.len()
            let sbox_value = *sbox.get(i).ok_or(StreamCipherError::ImplementationError)?;

            j = j
                .wrapping_add(usize::from(sbox_value))
                .wrapping_add(usize::from(key_value))
                .checked_rem_euclid(sbox.len())
                // ok_or can never happen since sbox.len() is always 256, not 0
                .ok_or(StreamCipherError::ImplementationError)?;

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
    fn encrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), StreamCipherError> {
        for x in buffer.iter_mut() {
            *x ^= self.byte_a4()?;
        }
        Ok(())
    }

    /// Decrypt bytes, storing the result in the same buffer.
    fn decrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), StreamCipherError> {
        self.encrypt_in_place(buffer)
    }

    /// Encrypt bytes, returning a vector
    fn encrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, StreamCipherError> {
        let mut encrypted_buffer = vec![0u8; buffer.len()];
        for (out_data, in_data) in encrypted_buffer.iter_mut().zip(buffer) {
            *out_data = in_data ^ self.byte_a4()?;
        }
        Ok(encrypted_buffer)
    }

    /// Decrypt bytes, returning a vector
    fn decrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, StreamCipherError> {
        self.encrypt(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_encrypt() {
        let mut a4_encrypt = AWCryptA4::new().unwrap();
        let mut a4_decrypt = a4_encrypt.clone();
        let data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();
        let encrypted_data = a4_encrypt.encrypt(&data).unwrap();
        let decrypted_data = a4_decrypt.decrypt(&encrypted_data).unwrap();

        assert!(
            data == decrypted_data,
            "Encrypting and decrypting did not produce the original data."
        );
    }

    #[test]
    pub fn test_encrypt_in_place() {
        let mut a4_encrypt = AWCryptA4::new().unwrap();
        let mut a4_inplace = a4_encrypt.clone();
        let mut data = (0u32..2048)
            .map(|x| (x & (u8::MAX as u32)) as u8)
            .collect::<Vec<u8>>();

        let encrypted_data = a4_encrypt.encrypt(&data).unwrap();

        a4_inplace.encrypt_in_place(data.as_mut_slice());

        assert!(
            encrypted_data == data,
            "Results of encrypt and encrypt_in_place differ."
        );
    }
}
