#[cfg_attr(feature = "python", pyo3::pyclass(frozen, eq, hash, module = "rumen"))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewMode {
    #[default]
    Normal,
    Mini,
    List,
    MiniList,
}

impl TryFrom<char> for ViewMode {
    type Error = &'static str;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            'v' => ViewMode::Normal,
            'm' => ViewMode::Mini,
            'd' => ViewMode::List,
            'D' => ViewMode::MiniList,
            _ => return Err("Invalid ViewMode"),
        })
    }
}
