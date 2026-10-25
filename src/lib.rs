//! Decode fumen v115 strings or URLs into complete, independent page snapshots.
//!
//! Fields and operation locations use the original fumen coordinates: rows run
//! from the top (0) to the garbage row (23), not SRS bottom-up coordinates.
//! Each page contains its field before placement and its resolved comment.
//!
//! ```
//! use rumen::{Block, Fumen};
//!
//! let fumen: Fumen = "v115@ghE8JeAgHvhAAAA".parse()?;
//! assert_eq!(fumen.pages.len(), 2);
//! assert_eq!(fumen.pages[0].field[22][5], Block::Garbage);
//! assert_eq!(fumen.pages[0].field, fumen.pages[1].field);
//! # Ok::<(), rumen::DecodeError>(())
//! ```

pub mod coding;

mod block;
mod color_scheme;
mod error;
mod facing;
mod location;
mod page;
mod quiz;
mod view_mode;

#[cfg(feature = "python")]
mod python;

pub use block::Block;
pub use color_scheme::ColorScheme;
pub use error::DecodeError;
pub use facing::Facing;
pub use location::Location;
pub use page::{Field, Flag, Page, Row};
pub use view_mode::ViewMode;

use quiz::Quiz;
use std::str::FromStr;

#[cfg_attr(feature = "python", pyo3::pyclass(frozen, eq, module = "rumen"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fumen {
    pub mode: ViewMode,
    pub version: u32,
    pub pages: Vec<Page>,
}

impl Fumen {
    pub fn new(mode: ViewMode, version: u32, pages: Vec<Page>) -> Self {
        Self {
            mode,
            version,
            pages,
        }
    }
}

impl Default for Fumen {
    fn default() -> Self {
        Self::new(ViewMode::Normal, 115, Vec::new())
    }
}

impl FromStr for Fumen {
    type Err = DecodeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse(input)
    }
}

struct Reader {
    digits: Vec<u8>,
    position: usize,
}

impl Reader {
    fn read(&mut self, count: usize) -> Result<u32, DecodeError> {
        let digits = self
            .digits
            .get(self.position..self.position + count)
            .ok_or(DecodeError::UnexpectedEnd {
                offset: self.position,
                needed: count,
            })?;
        let value = coding::base64::to_decimal(digits);
        self.position += count;
        Ok(value)
    }

    fn is_empty(&self) -> bool {
        self.position == self.digits.len()
    }

    fn comment(&mut self) -> Result<String, DecodeError> {
        let length = self.read(2)? as usize;
        let mut escaped = String::with_capacity(length);
        for _ in 0..length.div_ceil(4) {
            let mut value = self.read(5)?;
            for _ in 0..4 {
                if escaped.len() < length {
                    let digit = (value % 96) as u8;
                    escaped.push(
                        coding::base96::encode(digit)
                            .map_err(|_| DecodeError::InvalidCommentDigit(digit))?,
                    );
                }
                value /= 96;
            }
        }
        coding::unicode::decode(escaped).map_err(|_| DecodeError::InvalidUnicode)
    }
}

/// Parses a bare fumen or a viewer URL. Whitespace and `?` separators are ignored.
/// Versions other than v115 are rejected rather than interpreted as v115.
pub fn parse(input: &str) -> Result<Fumen, DecodeError> {
    let input = input.split('&').next().unwrap_or(input);
    let (start, mode, version) = input
        .char_indices()
        .find_map(|(start, character)| {
            let mode = ViewMode::try_from(character).ok()?;
            let header = input.get(start..start + 5)?.as_bytes();
            if header[4] != b'@' || !header[1..4].iter().all(u8::is_ascii_digit) {
                return None;
            }
            let version = header[1..4]
                .iter()
                .fold(0_u32, |value, digit| value * 10 + u32::from(digit - b'0'));
            Some((start + 5, mode, version))
        })
        .ok_or(DecodeError::InvalidHeader)?;
    if version != 115 {
        return Err(DecodeError::UnsupportedVersion(version));
    }
    let mut digits = Vec::with_capacity(input.len() - start);
    let mut offset = start;
    while let Some(&byte) = input.as_bytes().get(offset) {
        if let Ok(digit) = coding::base64::decode(byte as char) {
            digits.push(digit);
            offset += 1;
            continue;
        }
        let character = input[offset..].chars().next().unwrap();
        if character != '?' && !character.is_whitespace() {
            return Err(DecodeError::InvalidCharacter {
                character,
                byte_offset: offset,
            });
        }
        offset += character.len_utf8();
    }
    let mut reader = Reader {
        digits,
        position: 0,
    };
    let mut field = page::FieldState::new();
    let mut repeat = 0;
    let mut quiz: Option<Quiz> = None;
    let mut pages: Vec<Page> = Vec::new();

    while !reader.is_empty() {
        if repeat > 0 {
            repeat -= 1;
        } else {
            let mut index = 0;
            while index < 240 {
                let run = reader.read(2)?;
                let count = (run % 240 + 1) as usize;
                if index + count > 240 {
                    return Err(DecodeError::InvalidFieldRun { index, count });
                }
                let delta = (run / 240) as i16 - 8;
                field.add_run(index, count, delta)?;
                index += count;
                if run == 2159 {
                    repeat = reader.read(1)?;
                }
            }
        }

        let flag = Flag::try_from(reader.read(3)?).map_err(DecodeError::InvalidAction)?;
        let notes = if flag.note {
            let notes = reader.comment()?;
            quiz = Quiz::parse(&notes);
            notes
        } else {
            match &quiz {
                Some(quiz) => quiz.comment(),
                None => pages
                    .last()
                    .map_or_else(String::new, |page| page.notes.clone()),
            }
        };
        pages.push(Page::new(field.snapshot()?, flag, notes));

        if flag.lock && !reader.is_empty() {
            if let Some(quiz) = &mut quiz {
                quiz.advance(flag.mino);
            }
            field.advance(flag);
        }
    }
    if repeat > 0 {
        return Err(DecodeError::UnexpectedEnd {
            offset: reader.position,
            needed: 3,
        });
    }
    Ok(Fumen::new(mode, version, pages))
}
