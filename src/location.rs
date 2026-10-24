/// A fumen wire coordinate, with (0, 0) at the top-left and garbage at y=23.
/// I/O/S/Z anchors differ from their SRS rotation centers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    x: u8,
    y: u8,
}

impl Location {
    pub fn create(x: u8, y: u8) -> Result<Self, &'static str> {
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

impl TryFrom<[u8; 2]> for Location {
    type Error = &'static str;
    fn try_from([x, y]: [u8; 2]) -> Result<Self, Self::Error> {
        Location::create(x, y)
    }
}
