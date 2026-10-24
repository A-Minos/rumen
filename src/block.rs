#[cfg_attr(feature = "python", pyo3::pyclass(frozen, eq, hash, module = "rumen"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Block {
    Empty,
    I,
    L,
    O,
    Z,
    T,
    J,
    S,
    Garbage,
}

impl TryFrom<u8> for Block {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Block::Empty,
            1 => Block::I,
            2 => Block::L,
            3 => Block::O,
            4 => Block::Z,
            5 => Block::T,
            6 => Block::J,
            7 => Block::S,
            8 => Block::Garbage,
            _ => return Err("Invalid Block"),
        })
    }
}
