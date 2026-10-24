#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    GuideLine,
    Sega,
}

impl TryFrom<u8> for ColorScheme {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Sega,
            1 => Self::GuideLine,
            _ => return Err("Invalid ColorScheme"),
        })
    }
}
