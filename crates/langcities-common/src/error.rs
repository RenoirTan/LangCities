use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug)]
pub struct LcError<K: Debug + Send> {
    pub source: Option<Box<dyn Error + Send + Sync + 'static>>,
    pub kind: K,
}

impl<K: Debug + Send> LcError<K> {
    pub fn new(source: Option<Box<dyn Error + Send + Sync + 'static>>, kind: impl Into<K>) -> Self {
        let kind = kind.into();
        Self { source, kind }
    }
}

impl<K: Debug + Send> Display for LcError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LcError({:?})", self.kind)
    }
}

impl<K: Debug + Send> Error for LcError<K> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|e| e as &(dyn Error + 'static))
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
