#[derive(Clone, Debug)]
pub struct TreeContext {
    pub source: String,
}

impl TreeContext {
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        Self { source }
    }
}
