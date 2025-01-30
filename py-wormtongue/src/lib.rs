use pyo3::prelude::*;

#[pymodule]
fn wormtongue(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
