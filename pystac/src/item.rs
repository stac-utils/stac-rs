use pyo3::prelude::*;

#[pyclass]
pub struct Item(stac::Item);

#[pymethods]
impl Item {
    #[new]
    fn new(id: String) -> Item {
        Item(stac::Item::new(id))
    }

    #[getter]
    fn id(&self) -> &str {
        &self.0.id
    }

    #[setter]
    fn set_id(&mut self, id: String) {
        self.0.id = id;
    }
}
