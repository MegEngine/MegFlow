use pyo3::ffi;
use pyo3::Python;
use std::ffi::c_void;

lazy_static::lazy_static! {
    static ref VERSION: (u8,u8,u8) = {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let version = py.version_info();
            assert_eq!(version.major, 3);
            (version.major, version.minor, version.patch)
        })
    };
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CFrame {
    use_tracing: i32,
    previous: *mut CFrame,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyErrStackItem {
    pub exc_type: *mut ffi::PyObject,
    pub exc_value: *mut ffi::PyObject,
    pub exc_traceback: *mut ffi::PyObject,
    pub previous_item: *mut PyErrStackItem,
}

impl Default for PyErrStackItem {
    fn default() -> Self {
        PyErrStackItem {
            exc_type: std::ptr::null_mut(),
            exc_value: std::ptr::null_mut(),
            exc_traceback: std::ptr::null_mut(),
            previous_item: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PyThreadStateUnlimited3_10 {
    ob_base: ffi::PyObject,
    interp: *mut ffi::PyInterpreterState,
    frame: *mut ffi::PyFrameObject,
    recursion_depth: i32,
    recursion_headroom: i32,
    stackcheck_counter: i32,

    tracing: i32,
    cframe: *mut CFrame,

    c_profilefunc: *mut c_void,
    c_tracefunc: *mut c_void,
    c_profileobj: *mut ffi::PyObject,
    c_traceobj: *mut ffi::PyObject,
    curexc_type: *mut ffi::PyObject,
    curexc_value: *mut ffi::PyObject,
    curexc_traceback: *mut ffi::PyObject,
    exc_state: PyErrStackItem,
    exc_info: *mut PyErrStackItem,
    dict: *mut ffi::PyObject,

    gilstate_counter: i32,

    async_exc: *mut ffi::PyObject,
    thread_id: u64,

    trash_delete_nesting: i32,
    trash_delete_later: *mut ffi::PyObject,

    on_delete: *mut std::ffi::c_void,
    on_delete_data: *mut std::ffi::c_void,

    coroutine_origin_tracking_depth: i32,

    async_gen_firstiter: *mut ffi::PyObject,
    async_gen_finalizer: *mut ffi::PyObject,

    context: *mut ffi::PyObject,
    context_ver: u64,
    id: u64,
    root_cframe: CFrame,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PyThreadStateUnlimited3_789 {
    ob_base: ffi::PyObject,
    interp: *mut ffi::PyInterpreterState,
    frame: *mut ffi::PyFrameObject,
    recursion_depth: i32,
    overflowed: i8,
    recursion_critical: i8,
    stackcheck_counter: i32,
    tracing: i32,
    use_tracing: i32,
    c_profilefunc: *mut c_void,
    c_tracefunc: *mut c_void,
    c_profileobj: *mut ffi::PyObject,
    c_traceobj: *mut ffi::PyObject,
    curexc_type: *mut ffi::PyObject,
    curexc_value: *mut ffi::PyObject,
    curexc_traceback: *mut ffi::PyObject,
    exc_state: PyErrStackItem,
    exc_info: *mut PyErrStackItem,
    dict: *mut ffi::PyObject,

    gilstate_counter: i32,

    async_exc: *mut ffi::PyObject,
    thread_id: u64,

    trash_delete_nesting: i32,
    trash_delete_later: *mut ffi::PyObject,

    on_delete: *mut std::ffi::c_void,
    on_delete_data: *mut std::ffi::c_void,

    coroutine_origin_tracking_depth: i32,

    coroutine_wrapper: *mut ffi::PyObject,
    in_coroutine_wrapper: i32,

    async_gen_firstiter: *mut ffi::PyObject,
    async_gen_finalizer: *mut ffi::PyObject,

    context: *mut ffi::PyObject,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PyThreadStateUnlimited3_6 {
    ob_base: ffi::PyObject,
    interp: *mut ffi::PyInterpreterState,
    frame: *mut ffi::PyFrameObject,
    recursion_depth: i32,
    tracing: i32,
    use_tracing: i32,
    c_profilefunc: *mut c_void,
    c_tracefunc: *mut c_void,
    c_profileobj: *mut ffi::PyObject,
    c_traceobj: *mut ffi::PyObject,
    curexc_type: *mut ffi::PyObject,
    curexc_value: *mut ffi::PyObject,
    curexc_traceback: *mut ffi::PyObject,
    exc_type: *mut ffi::PyObject,
    exc_value: *mut ffi::PyObject,
    exc_traceback: *mut ffi::PyObject,
}

#[derive(Copy, Clone)]
pub struct PyThreadStateUnlimited {
    pub frame: *mut ffi::PyFrameObject,
    pub cframe: *mut CFrame,
    pub recursion_depth: i32,
    pub context: *mut ffi::PyObject,
    pub exc_state: PyErrStackItem,
    pub exc_info: *mut PyErrStackItem,
    pub exc_type: *mut ffi::PyObject,
    pub exc_value: *mut ffi::PyObject,
    pub exc_traceback: *mut ffi::PyObject,
}

impl Default for PyThreadStateUnlimited {
    fn default() -> Self {
        PyThreadStateUnlimited {
            frame: std::ptr::null_mut(),
            cframe: std::ptr::null_mut(),
            recursion_depth: 0,
            context: std::ptr::null_mut(),
            exc_state: Default::default(),
            exc_info: std::ptr::null_mut(),
            exc_type: std::ptr::null_mut(),
            exc_value: std::ptr::null_mut(),
            exc_traceback: std::ptr::null_mut(),
        }
    }
}

pub fn store(ts: *mut ffi::PyThreadState) -> PyThreadStateUnlimited {
    match VERSION.1 {
        6 => {
            let ts = ts as *mut PyThreadStateUnlimited3_6;
            unsafe {
                let unlimited = PyThreadStateUnlimited {
                    frame: (*ts).frame,
                    recursion_depth: (*ts).recursion_depth,
                    exc_type: (*ts).exc_type,
                    exc_value: (*ts).exc_value,
                    exc_traceback: (*ts).exc_traceback,
                    ..Default::default()
                };
                (*ts).frame = std::ptr::null_mut();
                (*ts).recursion_depth = 0;
                (*ts).exc_type = std::ptr::null_mut();
                (*ts).exc_value = std::ptr::null_mut();
                (*ts).exc_traceback = std::ptr::null_mut();
                unlimited
            }
        }
        7 | 8 | 9 => {
            let ts = ts as *mut PyThreadStateUnlimited3_789;
            unsafe {
                let unlimited = PyThreadStateUnlimited {
                    frame: (*ts).frame,
                    recursion_depth: (*ts).recursion_depth,
                    context: (*ts).context,
                    exc_state: (*ts).exc_state,
                    exc_info: (*ts).exc_info,
                    ..Default::default()
                };
                (*ts).frame = std::ptr::null_mut();
                (*ts).recursion_depth = 0;
                (*ts).context = std::ptr::null_mut();
                (*ts).exc_state = Default::default();
                (*ts).exc_info = std::ptr::addr_of_mut!((*ts).exc_state);
                unlimited
            }
        }
        10 => {
            let ts = ts as *mut PyThreadStateUnlimited3_10;
            unsafe {
                let unlimited = PyThreadStateUnlimited {
                    frame: (*ts).frame,
                    cframe: (*ts).cframe,
                    recursion_depth: (*ts).recursion_depth,
                    context: (*ts).context,
                    exc_state: (*ts).exc_state,
                    exc_info: (*ts).exc_info,
                    ..Default::default()
                };
                (*ts).frame = std::ptr::null_mut();
                (*ts).recursion_depth = 0;
                (*ts).context = std::ptr::null_mut();
                (*ts).exc_state = Default::default();
                (*ts).exc_info = std::ptr::addr_of_mut!((*ts).exc_state);
                (*ts).cframe = std::ptr::addr_of_mut!((*ts).root_cframe);
                unlimited
            }
        }
        _ => unimplemented!(),
    }
}

pub fn restore(limited: *mut ffi::PyThreadState, unlimited: &PyThreadStateUnlimited) {
    match VERSION.1 {
        6 => {
            let ts = limited as *mut PyThreadStateUnlimited3_6;
            unsafe {
                (*ts).frame = unlimited.frame;
                (*ts).recursion_depth = unlimited.recursion_depth;
                (*ts).exc_type = unlimited.exc_type;
                (*ts).exc_value = unlimited.exc_value;
                (*ts).exc_traceback = unlimited.exc_traceback;
            }
        }
        7 | 8 | 9 => {
            let ts = limited as *mut PyThreadStateUnlimited3_789;
            unsafe {
                (*ts).frame = unlimited.frame;
                (*ts).recursion_depth = unlimited.recursion_depth;
                (*ts).context = unlimited.context;
                (*ts).exc_state = unlimited.exc_state;
                (*ts).exc_info = unlimited.exc_info;
            }
        }
        10 => {
            let ts = limited as *mut PyThreadStateUnlimited3_10;
            unsafe {
                (*ts).frame = unlimited.frame;
                (*ts).cframe = unlimited.cframe;
                (*ts).recursion_depth = unlimited.recursion_depth;
                (*ts).context = unlimited.context;
                (*ts).exc_state = unlimited.exc_state;
                (*ts).exc_info = unlimited.exc_info;
            }
        }
        _ => unimplemented!(),
    }
}
