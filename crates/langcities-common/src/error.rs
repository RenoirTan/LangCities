use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug)]
pub struct LcError<K: Debug> {
    pub source: Option<Box<dyn Error>>,
    pub kind: K,
}

impl<K: Debug> LcError<K> {
    pub fn new(source: Option<Box<dyn Error>>, kind: impl Into<K>) -> Self {
        let kind = kind.into();
        Self { source, kind }
    }
}

impl<K: Debug> Display for LcError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LcError({:?})", self.kind)
    }
}

impl<K: Debug> Error for LcError<K> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
