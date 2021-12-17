/**
 * \file flow-rs/src/helper.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use numpy::array::PyArrayDyn;
use numpy::Element;
use pyo3::prelude::*;
use std::ops::Deref;

pub struct SliceGuard<'a, T>
where
    T: Element,
{
    slice: &'a [T],
    _ref: PyObject,
}

impl<'a, T> Deref for SliceGuard<'a, T>
where
    T: Element,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

pub fn uget_slice<'a, T: Element>(py: Python, pyobject: &PyAny) -> PyResult<SliceGuard<'a, T>> {
    let array: &PyArrayDyn<T> = pyobject.extract()?;
    unsafe {
        let slice = array
            .as_slice()
            .map_err(|_| pyo3::exceptions::PyTypeError::new_err("not contiguous"))?;
        let slice = std::slice::from_raw_parts(slice.as_ptr(), slice.len());
        Ok(SliceGuard {
            slice,
            _ref: pyobject.into_py(py),
        })
    }
}
