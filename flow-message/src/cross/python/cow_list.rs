use super::*;
use pyo3::{PyIterProtocol, PySequenceProtocol};

#[pyproto]
impl PySequenceProtocol for PyCowList {
    fn __len__(&self) -> usize {
        self.as_ref().len()
    }

    fn __setitem__(&mut self, idx: isize, value: PyObject) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let data = to_data(py, value)?;
        let len = self.as_ref().len();
        let idx = cvt_idx(len, idx);
        self.as_mut().set(idx, data);
        Ok(())
    }

    fn __getitem__(&self, idx: isize) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let len = self.as_ref().len();
        let idx = cvt_idx(len, idx);
        if let Some(data) = self.as_ref().get(idx) {
            from_data(py, data)
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err("Idx out of bound"))
        }
    }

    fn __delitem__(&mut self, idx: isize) -> PyResult<()> {
        let len = self.as_ref().len();
        let idx = cvt_idx(len, idx);
        self.as_mut().remove(idx);
        Ok(())
    }

    fn __concat__(&self, other: Py<PyCowList>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let other = other.borrow(py);
        let newlist = self
            .as_ref()
            .iter()
            .chain(other.as_ref().iter())
            .cloned()
            .collect::<CowList>();
        Py::new(
            py,
            PyCowList {
                inner: Some(newlist),
            },
        )
        .map(|list| list.to_object(py))
    }

    fn __repeat__(&self, count: isize) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut newlist = self.as_ref().clone();
        for _ in 0..count - 1 {
            for data in self.as_ref() {
                newlist.push_back(data.clone());
            }
        }
        Py::new(
            py,
            PyCowList {
                inner: Some(newlist),
            },
        )
        .map(|list| list.to_object(py))
    }
}

#[pyproto]
impl PyIterProtocol for PyCowList {
    fn __iter__(this: PyRefMut<Self>) -> PyResult<super::iterator::PyObjectIterator> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut elements = std::vec::Vec::new();
        for element in this.as_ref().iter() {
            elements.push(from_data(py, &element.clone())?);
        }

        Ok(super::iterator::PyObjectIterator::new(elements.into_iter()))
    }
}
