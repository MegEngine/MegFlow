use pyo3::Python;
use std::any::{Any, TypeId};
use std::sync::Arc;

pub trait DowncastArc {
    fn into_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<T> DowncastArc for T
where
    T: 'static + Send + Sync,
{
    fn into_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

pub trait Resource: Any + DowncastArc + Send + Sync {
    fn to_python(&self, py: Python) -> pyo3::PyObject;
}

impl dyn Resource {
    pub fn is<T: Resource>(&self) -> bool {
        let t = TypeId::of::<T>();
        let boxed = self.type_id();

        t == boxed
    }
    #[inline]
    pub fn downcast_arc<T: Resource>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>>
    where
        Self: Send + Sync,
    {
        if self.is::<T>() {
            Ok(DowncastArc::into_arc_any(self).downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }
}

pub type AnyResource = Arc<dyn Resource>;
