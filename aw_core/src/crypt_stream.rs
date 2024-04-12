use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum StreamCipherError {
    KeyTooShort,
    ImplementationError,
}

impl Error for StreamCipherError {}

impl fmt::Display for StreamCipherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamCipherError::KeyTooShort => write!(f, "Key is too short"),
            StreamCipherError::ImplementationError => {
                write!(
                    f,
                    "Something went wrong in the internal implementation of the stream cipher"
                )
            }
        }
    }
}

pub trait AWCryptStream: Sized {
    /// Create a new stream cipher with a new key.
    fn new() -> Result<Self, StreamCipherError>;

    /// Create an stream cipher using an existing key.
    fn from_key(key: &[u8]) -> Result<Self, StreamCipherError>;

    /// Get the initial key value used to set up the cipher
    fn get_initial_random_buffer(&self) -> Vec<u8>;

    /// Encrypt bytes, storing the result in the same buffer.
    fn encrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), StreamCipherError>;

    /// Decrypt bytes, storing the result in the same buffer.
    fn decrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), StreamCipherError>;

    /// Encrypt bytes, returning a vector
    fn encrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, StreamCipherError>;

    /// Decrypt bytes, returning a vector
    fn decrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, StreamCipherError>;
}
