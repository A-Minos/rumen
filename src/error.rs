use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidHeader,
    UnsupportedVersion(u32),
    InvalidCharacter {
        character: char,
        byte_offset: usize,
    },
    /// `offset` counts base64 symbols in the payload, excluding separators.
    UnexpectedEnd {
        offset: usize,
        needed: usize,
    },
    InvalidFieldRun {
        index: usize,
        count: usize,
    },
    InvalidBlock {
        index: usize,
        value: i16,
    },
    InvalidCommentDigit(u8),
    InvalidAction(&'static str),
    UndefinedCell {
        index: usize,
    },
    InvalidUnicode,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader => {
                write!(formatter, "expected a v115@, m115@, d115@ or D115@ fumen")
            }
            Self::UnsupportedVersion(version) => {
                write!(
                    formatter,
                    "unsupported fumen version {version}; only v115 is supported"
                )
            }
            Self::InvalidCharacter {
                character,
                byte_offset,
            } => {
                write!(
                    formatter,
                    "invalid base64 character {character:?} at byte {byte_offset}"
                )
            }
            Self::UnexpectedEnd { offset, needed } => {
                write!(
                    formatter,
                    "truncated fumen: expected {needed} symbols at payload offset {offset}"
                )
            }
            Self::InvalidFieldRun { index, count } => {
                write!(
                    formatter,
                    "field run of {count} cells at index {index} exceeds 240 cells"
                )
            }
            Self::InvalidBlock { index, value } => {
                write!(
                    formatter,
                    "field delta produces invalid block {value} at index {index}"
                )
            }
            Self::InvalidCommentDigit(digit) => {
                write!(
                    formatter,
                    "comment digit {digit} is not in the printable ASCII alphabet"
                )
            }
            Self::InvalidAction(reason) => write!(formatter, "invalid action: {reason}"),
            Self::UndefinedCell { index } => {
                write!(
                    formatter,
                    "field cell {index} is undefined after an off-board placement"
                )
            }
            Self::InvalidUnicode => {
                write!(formatter, "comment contains an unpaired UTF-16 surrogate")
            }
        }
    }
}

impl std::error::Error for DecodeError {}
