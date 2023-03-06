//use pyo3::prelude::*;

pub mod defaults;
pub mod polymer;
pub mod mass;
pub mod mzml;
// Formats the sum of two numbers as string.
//#[pyfunction]
//fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//    Ok((a + b).to_string())
//}

// A Python module implemented in Rust.
//#[pymodule]
//fn mzsniffer(_py: Python, m: &PyModule) -> PyResult<()> {
//    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
//    Ok(())
//}
