use aw_core::ReasonCode;
use std::io;
use thiserror::Error;

/// Comprehensive error type for the ActiveWorlds SDK
#[derive(Error, Debug)]
pub enum SdkError {
    /// Network-related errors (connection failures, timeouts, etc.)
    #[error("Network error: {0}")]
    Network(#[from] io::Error),

    /// Protocol-level errors (malformed packets, unexpected responses)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Authentication and authorization errors
    #[error("ActiveWorlds error: {0:?}")]
    ActiveWorldsError(ReasonCode),

    /// Cryptographic errors (key exchange, encryption/decryption failures)
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Timeout errors
    #[error("Operation timed out")]
    Timeout,

    /// Connection state errors (trying to use a disconnected connection, etc.)
    #[error("Connection state error: {0}")]
    ConnectionState(String),

    /// Server returned an error reason code
    #[error("Server error: {0:?}")]
    ServerError(ReasonCode),

    /// Missing required data in packet
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Not logged in when login is required
    #[error("Not logged in")]
    NotLoggedIn,

    /// Already connected when trying to connect again
    #[error("Already connected")]
    AlreadyConnected,

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Not connected to a world
    #[error("Not connected to a world")]
    NotConnectedToWorld,
}

impl SdkError {
    /// Helper to create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::Protocol(msg.into())
    }

    /// Helper to create a crypto error
    pub fn crypto(msg: impl Into<String>) -> Self {
        Self::Crypto(msg.into())
    }

    /// Helper to create a connection state error
    pub fn connection_state(msg: impl Into<String>) -> Self {
        Self::ConnectionState(msg.into())
    }

    /// Helper to create a missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField(field.into())
    }

    /// Helper to create an invalid parameter error
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        Self::InvalidParameter(msg.into())
    }
}

/// Type alias for SDK results
pub type SdkResult<T> = Result<T, SdkError>;
