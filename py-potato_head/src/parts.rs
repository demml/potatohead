use ::potato_head::Mouth;
use pyo3::prelude::*;

#[pymodule]
pub fn parts(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mouth>()?;
    Ok(())
}
