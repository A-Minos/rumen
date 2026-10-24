use super::{Block, ColorScheme, DecodeError, Facing, Location};

pub type Row = [Block; 10];
/// Rows 0..23 are the playfield from top to bottom; row 23 is garbage.
pub type Field = [Row; 24];

#[cfg_attr(
    feature = "python",
    pyo3::pyclass(frozen, get_all, eq, module = "rumen")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// The field before this page's operation and events are applied.
    pub field: Field,
    pub flag: Flag,
    /// The visible comment, including inheritance and quiz progression.
    pub notes: String,
}

impl Page {
    pub fn new(field: Field, flag: Flag, notes: String) -> Self {
        Self { field, flag, notes }
    }
}

#[cfg_attr(
    feature = "python",
    pyo3::pyclass(frozen, get_all, eq, module = "rumen")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flag {
    pub mino: Block,
    pub face: Option<Facing>,
    pub location: Option<Location>,
    pub up: bool,
    pub mirror: bool,
    pub color_scheme: ColorScheme,
    /// Whether a new comment was encoded on this page.
    pub note: bool,
    pub lock: bool,
}

impl TryFrom<u32> for Flag {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let mino = Block::try_from((value % 8) as u8)?;
        let face = if mino == Block::Empty {
            None
        } else {
            Some(Facing::try_from((value / 8 % 4) as u8)?)
        };
        let location = if mino == Block::Empty {
            None
        } else {
            Some(Location::try_from((value / 32 % 240) as u8)?)
        };
        let flag = value / (32 * 240);
        Ok(Self {
            mino,
            face,
            location,
            up: flag & 1 != 0,
            mirror: flag & 2 != 0,
            color_scheme: ColorScheme::try_from(((flag >> 2) & 1) as u8)?,
            note: flag & 8 != 0,
            lock: flag & 16 == 0,
        })
    }
}

#[cfg_attr(feature = "python", pyo3::pymethods)]
impl Flag {
    /// The four occupied cells in top-down field coordinates, or no active mino.
    /// Coordinates may extend outside the playfield; this is not a game-legality check.
    pub fn positions(&self) -> Option<[(i16, i16); 4]> {
        let face = self.face?;
        let location = self.location?;
        let horizontal = matches!(face, Facing::Up | Facing::Down);
        let offsets = match self.mino {
            Block::Empty | Block::Garbage => return None,
            Block::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Block::I if horizontal => [(-1, 0), (0, 0), (1, 0), (2, 0)],
            Block::I => [(0, -1), (0, 0), (0, 1), (0, 2)],
            Block::S if horizontal => [(-1, 1), (0, 1), (0, 0), (1, 0)],
            Block::S => [(-1, -1), (-1, 0), (0, 0), (0, 1)],
            Block::Z if horizontal => [(-1, 0), (0, 0), (0, 1), (1, 1)],
            Block::Z => [(1, -1), (1, 0), (0, 0), (0, 1)],
            mino => {
                let spawn = match mino {
                    Block::T => [(-1, 0), (0, 0), (1, 0), (0, -1)],
                    Block::J => [(-1, 0), (0, 0), (1, 0), (-1, -1)],
                    Block::L => [(-1, 0), (0, 0), (1, 0), (1, -1)],
                    _ => unreachable!(),
                };
                spawn.map(|(x, y)| match face {
                    Facing::Up => (x, y),
                    Facing::Right => (-y, x),
                    Facing::Down => (-x, -y),
                    Facing::Left => (y, -x),
                })
            }
        };
        Some(offsets.map(|(x, y)| (x + i16::from(location.x()), y + i16::from(location.y()))))
    }
}

// Wire-state behavior follows tetris-fumen 1.1.3 (MIT; see LICENSES/tetris-fumen.txt).
pub(crate) struct FieldState {
    // The reference uses a bottom-up array. Off-board locks can extend it;
    // its length and sparse cells affect subsequent clears, even above row 22.
    blocks: Vec<Option<Block>>,
    garbage: Row,
}

impl FieldState {
    pub(crate) fn new() -> Self {
        Self {
            blocks: vec![Some(Block::Empty); 230],
            garbage: [Block::Empty; 10],
        }
    }

    pub(crate) fn add(&mut self, index: usize, delta: i16) -> Result<(), DecodeError> {
        let cell = if index < 230 {
            self.blocks
                .get_mut((22 - index / 10) * 10 + index % 10)
                .and_then(Option::as_mut)
                .ok_or(DecodeError::UndefinedCell { index })?
        } else {
            &mut self.garbage[index - 230]
        };
        let value = *cell as i16 + delta;
        *cell = u8::try_from(value)
            .ok()
            .and_then(|value| Block::try_from(value).ok())
            .ok_or(DecodeError::InvalidBlock { index, value })?;
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> Result<Field, DecodeError> {
        let mut field = [[Block::Empty; 10]; 24];
        for (index, cell) in field[..23].iter_mut().flatten().enumerate() {
            *cell = self
                .blocks
                .get((22 - index / 10) * 10 + index % 10)
                .copied()
                .flatten()
                .ok_or(DecodeError::UndefinedCell { index })?;
        }
        field[23] = self.garbage;
        Ok(field)
    }

    pub(crate) fn advance(&mut self, flag: Flag) {
        if let Some(positions) = flag.positions() {
            for (x, y) in positions {
                let index = x + (22 - y) * 10;
                // Negative JavaScript array properties do not survive concat().
                if let Ok(index) = usize::try_from(index) {
                    if self.blocks.len() <= index {
                        self.blocks.resize(index + 1, None);
                    }
                    self.blocks[index] = Some(flag.mino);
                }
            }
        }

        let before_clear = self.blocks.clone();
        // Keep the reference's fractional row boundaries for extended arrays.
        let mut row = before_clear.len() as f64 / 10.0 - 1.0;
        while row >= 0.0 {
            let start = ((row * 10.0) as usize).min(before_clear.len());
            let end = (((row + 1.0) * 10.0) as usize).min(before_clear.len());
            if before_clear[start..end]
                .iter()
                .all(|cell| *cell != Some(Block::Empty))
            {
                self.blocks
                    .drain(start.min(self.blocks.len())..end.min(self.blocks.len()));
                self.blocks.extend([Some(Block::Empty); 10]);
            }
            row -= 1.0;
        }

        if flag.up {
            self.blocks.splice(0..0, self.garbage.map(Some));
            self.blocks.truncate(230);
            self.garbage.fill(Block::Empty);
        }
        if flag.mirror {
            for row in self.blocks.chunks_mut(10) {
                row.reverse();
            }
        }
    }
}
