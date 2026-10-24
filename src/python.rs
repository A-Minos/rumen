use crate::{Block, ColorScheme, Facing, Flag, Fumen, Location, Page, ViewMode};
use pyo3::{exceptions::PyValueError, prelude::*};

pyo3::create_exception!(rumen, DecodeError, PyValueError);

/// Decode a v115 fumen string or viewer URL.
///
/// Objects have read-only attributes; page and field lists are independent copies.
/// Coordinates use the fumen top-left origin, with the garbage row at y=23.
/// Raises DecodeError if the input cannot be decoded.
#[pyfunction]
fn parse(py: Python<'_>, input: &str) -> PyResult<Fumen> {
    py.detach(|| crate::parse(input))
        .map_err(|error| DecodeError::new_err(error.to_string()))
}

/// Fumen v115 decoding backed by Rust.
#[pymodule(gil_used = false)]
fn rumen(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(parse, module)?)?;
    module.add_class::<Fumen>()?;
    module.add_class::<Page>()?;
    module.add_class::<Flag>()?;
    module.add_class::<Location>()?;
    module.add_class::<Block>()?;
    module.add_class::<Facing>()?;
    module.add_class::<ColorScheme>()?;
    module.add_class::<ViewMode>()?;
    module.add("DecodeError", module.py().get_type::<DecodeError>())?;
    Ok(())
}
