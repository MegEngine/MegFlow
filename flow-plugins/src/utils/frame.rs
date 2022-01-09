use glider_service::mgc;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(name = "Frame")]
pub struct PyFrame {
    inner: mgc::Frame,
    meta: Py<PyDict>,
}

impl PyFrame {
    pub fn new(inner: mgc::Frame, meta: Py<PyDict>) -> Self {
        PyFrame { inner, meta }
    }
}

#[pymethods]
impl PyFrame {
    #[getter(frame_id)]
    fn frame_id(&self) -> usize {
        self.inner.frame_id as usize
    }
    fn get_meta(&self) -> &Py<PyDict> {
        &self.meta
    }
    #[getter(data)]
    fn get_data(&self) -> usize {
        self.inner.data as usize
    }

    #[getter(width)]
    fn get_width(&self) -> usize {
        self.inner.width as usize
    }

    #[getter(height)]
    fn get_height(&self) -> usize {
        self.inner.height as usize
    }

    #[getter(original_width)]
    fn original_width(&self) -> usize {
        self.inner.original_width as usize
    }

    #[getter(original_height)]
    fn original_height(&self) -> usize {
        self.inner.original_height as usize
    }

    fn is_host(&self) -> bool {
        self.inner.device.dev_type == mgc::DeviceType::CPU
    }

    fn is_device(&self) -> bool {
        !self.is_host()
    }

    #[getter(dev_id)]
    fn get_dev_id(&self) -> Option<isize> {
        if self.is_device() {
            Some(self.inner.device.dev_id as isize)
        } else {
            None
        }
    }

    #[getter(pts)]
    fn get_pts(&self) -> isize {
        self.inner.pts as isize
    }

    #[getter(pixel_format)]
    fn get_pixel_format(&self) -> &str {
        let fmt = self.inner.fmt;
        if fmt == mgc::PixelFormat::YUV420P {
            "yuv420p"
        } else if fmt == mgc::PixelFormat::NV12 {
            "nv12"
        } else if fmt == mgc::PixelFormat::NV21 {
            "nv21"
        } else if fmt == mgc::PixelFormat::YUVJ420P {
            "yuvj420p"
        } else if fmt == mgc::PixelFormat::BGR24_PACKED {
            "bgr24_packed"
        } else if fmt == mgc::PixelFormat::BGR24_PLANAR {
            "bgr24_planar"
        } else if fmt == mgc::PixelFormat::RGB24_PACKED {
            "rgb24_packed"
        } else if fmt == mgc::PixelFormat::RGB24_PLANAR {
            "rgb24_planar"
        } else {
            "unknow"
        }
    }
}
