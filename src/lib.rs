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

mod block;
mod coding;
mod color_scheme;
mod decode;
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
pub use decode::parse;
pub use error::DecodeError;
pub use facing::Facing;
pub use location::Location;
pub use page::{Field, Flag, Page, Row};
pub use view_mode::ViewMode;

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
