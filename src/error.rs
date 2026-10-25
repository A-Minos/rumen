use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecodeError {
    #[error("expected a v115@, m115@, d115@ or D115@ fumen")]
    InvalidHeader,
    #[error("unsupported fumen version {0}; only v115 is supported")]
    UnsupportedVersion(u32),
    #[error("invalid base64 character {character:?} at byte {byte_offset}")]
    InvalidCharacter { character: char, byte_offset: usize },
    /// `offset` counts base64 symbols in the payload, excluding separators.
    #[error("truncated fumen: expected {needed} symbols at payload offset {offset}")]
    UnexpectedEnd { offset: usize, needed: usize },
    #[error("field run of {count} cells at index {index} exceeds 240 cells")]
    InvalidFieldRun { index: usize, count: usize },
    #[error("field delta produces invalid block {value} at index {index}")]
    InvalidBlock { index: usize, value: i16 },
    #[error("comment digit {0} is not in the printable ASCII alphabet")]
    InvalidCommentDigit(u8),
    #[error("invalid action: {0}")]
    InvalidAction(&'static str),
    #[error("field cell {index} is undefined after an off-board placement")]
    UndefinedCell { index: usize },
    #[error("comment contains an unpaired UTF-16 surrogate")]
    InvalidUnicode,
}
