/// A fumen wire coordinate, with (0, 0) at the top-left and garbage at y=23.
/// I/O/S/Z anchors differ from their SRS rotation centers.
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(frozen, get_all, eq, hash, module = "rumen")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    x: u8,
    y: u8,
}

impl Location {
    pub fn new(x: u8, y: u8) -> Result<Self, &'static str> {
        if x < 10 && y < 24 {
            Ok(Location { x, y })
        } else {
            Err("Invalid Location")
        }
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }
}

impl TryFrom<u8> for Location {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 240 {
            Ok(Location {
                x: value % 10,
                y: value / 10,
            })
        } else {
            Err("Invalid Location")
        }
    }
}
