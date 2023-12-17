use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum StreamKeyError {
    TooShort,
}

impl Error for StreamKeyError {}

impl fmt::Display for StreamKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamKeyError::TooShort => write!(f, "Key is too short"),
        }
    }
}

pub trait AWCryptStream: Default {
    /// Create a new stream cipher with a new key.
    fn new() -> Self;

    /// Create an stream cipher using an existing key.
    fn from_key(key: &[u8]) -> Result<Self, StreamKeyError>;

    /// Get the initial key value used to set up the cipher
    fn get_initial_random_buffer(&self) -> Vec<u8>;

    /// Encrypt bytes, storing the result in the same buffer.
    fn encrypt_in_place(&mut self, buffer: &mut [u8]);

    /// Decrypt bytes, storing the result in the same buffer.
    fn decrypt_in_place(&mut self, buffer: &mut [u8]);

    /// Encrypt bytes, returning a vector
    fn encrypt(&mut self, buffer: &[u8]) -> Vec<u8>;

    /// Decrypt bytes, returning a vector
    fn decrypt(&mut self, buffer: &[u8]) -> Vec<u8>;
}
