use crate::{Block, ColorScheme, Facing, Flag, Fumen, Location, Page, ViewMode};
use pyo3::{
    exceptions::{PyIndexError, PyValueError},
    prelude::*,
    types::{PyIterator, PyList},
};

pyo3::create_exception!(rumen, DecodeError, PyValueError);

#[pymethods]
impl Fumen {
    #[getter]
    fn mode(&self) -> ViewMode {
        self.mode
    }

    #[getter]
    fn version(&self) -> u32 {
        self.version
    }

    fn __len__(&self) -> usize {
        self.pages.len()
    }

    fn __iter__<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyIterator>> {
        // Calling iter(slf) would recurse; use CPython's lazy sequence iterator.
        // SAFETY: slf is a live, attached sequence. PySeqIter_New returns a new
        // reference (which retains slf), or null with an exception set.
        let iterator = unsafe {
            Bound::from_owned_ptr_or_err(slf.py(), pyo3::ffi::PySeqIter_New(slf.as_ptr()))?
        };
        Ok(iterator.cast_into::<PyIterator>()?)
    }

    /// Return one page snapshot without copying the whole page list.
    fn __getitem__(&self, index: isize) -> PyResult<Page> {
        let index = if index < 0 {
            index + self.pages.len() as isize
        } else {
            index
        };
        self.pages
            .get(index as usize)
            .cloned()
            .ok_or_else(|| PyIndexError::new_err("page index out of range"))
    }

    /// Copy all pre-action fields into page/row/column integer lists (codes 0..8).
    fn to_matrices<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let mut matrices = Vec::with_capacity(self.pages.len());
        for page in &self.pages {
            matrices.push(page.to_matrix(py)?);
        }
        PyList::new(py, matrices)
    }
}

#[pymethods]
impl Page {
    #[getter]
    fn flag(&self) -> Flag {
        self.flag
    }

    /// The resolved comment, including inheritance and quiz progression.
    #[getter]
    fn notes(&self) -> &str {
        &self.notes
    }

    /// Copy the 24x10 pre-action field into integer lists; row 23 is garbage.
    fn to_matrix<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let mut rows = Vec::with_capacity(self.field.len());
        for row in &self.field {
            rows.push(PyList::new(py, row.iter().map(|block| *block as u8))?);
        }
        PyList::new(py, rows)
    }
}

/// Decode a v115 fumen string or viewer URL.
///
/// Index or iterate the result for page snapshots; attributes are read-only.
/// Use Page.to_matrix() or Fumen.to_matrices() for explicit numeric field copies.
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
