/// Fumen wire directions: down=0, right=1, up=2 (spawn), left=3.
#[cfg_attr(feature = "python", pyo3::pyclass(frozen, eq, hash, module = "rumen"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facing {
    Down,
    Right,
    Up,
    Left,
}

impl TryFrom<u8> for Facing {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Facing::Down,
            1 => Facing::Right,
            2 => Facing::Up,
            3 => Facing::Left,
            _ => return Err("Invalid Facing"),
        })
    }
}
