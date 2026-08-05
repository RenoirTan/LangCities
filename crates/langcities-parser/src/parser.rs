use langcities_dsl::tree::Tree;

use crate::{ParserError, Preparer, Transferer};

pub struct Parser {
    pub transferer: Transferer,
}

impl Parser {
    pub fn new(transferer: impl Into<Transferer>) -> Self {
        let transferer = transferer.into();
        Self { transferer }
    }

    pub fn from_preparer(preparer: impl Into<Preparer>) -> Self {
        let transferer = Transferer::new(preparer);
        Self::new(transferer)
    }

    pub fn from_source(source: impl Into<String>) -> Result<Self, ParserError> {
        let preparer = Preparer::from_source(source)?;
        Ok(Self::from_preparer(preparer))
    }

    pub fn parse<'p>(&'p mut self) -> Result<&'p Tree, ParserError> {
        self.transferer.transfer()?;
        Ok(&self.transferer.preparer.tree_builder.tree)
    }
}
