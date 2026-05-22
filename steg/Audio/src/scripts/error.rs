use thiserror::Error;

#[derive(Debug, Error)]
pub enum StegoError {
    #[error("payload too large: need {needed} samples, have {available}")]
    PayloadTooLarge { needed: usize, available: usize },

    #[error("no steganography header found in audio")]
    NoHeaderFound,

    #[error("unsupported WAV format: {0}")]
    UnsupportedFormat(String),

    #[error("embedded payload is not valid UTF-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("WAV error: {0}")]
    Wav(#[from] hound::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
