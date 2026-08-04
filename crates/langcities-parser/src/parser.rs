use langcities_dsl::tree::TreeBuilder;
use tree_sitter::{Parser as TSParser, Tree as TSTree};
use tree_sitter_lcdcdsl::LANGUAGE;

use crate::{ParserError, ParserErrorKind};

pub struct Parser {
    pub tree_builder: TreeBuilder,
    pub ts_parser: TSParser,
    pub raw_tree: TSTree,
}

impl Parser {
    pub fn new<B, P>(tree_builder: B, ts_parser: P) -> Result<Self, ParserError>
    where
        B: Into<TreeBuilder>,
        P: Into<TSParser>,
    {
        let (tree_builder, mut ts_parser) = (tree_builder.into(), ts_parser.into());
        let source = &tree_builder.tree.context.source;
        let raw_tree = ts_parser
            .parse(source, None)
            .ok_or_else(|| ParserError::new(None, ParserErrorKind::InvalidSource))?;
        Ok(Self {
            tree_builder,
            ts_parser,
            raw_tree,
        })
    }

    pub fn from_source(source: impl Into<String>) -> Result<Self, ParserError> {
        let tree_builder = TreeBuilder::new(source);
        let mut ts_parser = TSParser::new();
        ts_parser
            .set_language(&LANGUAGE.into())
            .map_err(|e| ParserError::new(Some(Box::new(e)), ParserErrorKind::BadInitialization))?;
        Self::new(tree_builder, ts_parser)
    }

    pub fn transfer(&mut self) -> Result<(), ParserError> {
        // TODO: Complete Parser::transfer
        Ok(())
    }
}
