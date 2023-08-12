use std::borrow::Cow;

#[cfg(feature = "py")]
use pyo3::FromPyObject;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LayerIdentifier {
    Ordinal(i32),
    Named(Cow<'static, str>),
}

impl Default for LayerIdentifier {
    fn default() -> Self {
        Self::Ordinal(0)
    }
}

impl From<&'_ str> for LayerIdentifier {
    fn from(value: &'_ str) -> Self {
        Self::Named(value.to_owned().into())
    }
}

impl From<i32> for LayerIdentifier {
    fn from(value: i32) -> Self {
        Self::Ordinal(value)
    }
}

#[cfg(feature = "py")]
impl FromPyObject<'_> for LayerIdentifier {
    fn extract(ob: &'_ pyo3::PyAny) -> pyo3::PyResult<Self> {
        match ob.extract::<&str>() {
            Ok(str) => Ok(LayerIdentifier::from(str)),
            Err(_) => ob.extract::<i32>().map(LayerIdentifier::from),
        }
    }
}
