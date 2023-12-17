fn main() {
    #[cfg(feature = "stream_cipher_rc4")]
    let stream_cipher_rc4 = true;
    #[cfg(not(feature = "stream_cipher_rc4"))]
    let stream_cipher_rc4 = false;

    #[cfg(feature = "stream_cipher_aes")]
    let stream_cipher_aes = true;
    #[cfg(not(feature = "stream_cipher_aes"))]
    let stream_cipher_aes = false;

    if stream_cipher_rc4 && stream_cipher_aes {
        panic!("Features 'stream_cipher_rc4' and 'stream_cipher_aes' cannot be enabled at the same time");
    }

    if !stream_cipher_rc4 && !stream_cipher_aes {
        panic!("One of 'stream_cipher_rc4' or 'stream_cipher_aes' must be enabled");
    }
}
