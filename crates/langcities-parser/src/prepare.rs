use langcities_dsl::{
    node::{BinaryOp, StringLiteralKind},
    tree::TreeBuilder,
};
use tree_sitter::{Parser as TSParser, Tree as TSTree};
use tree_sitter_lcdcdsl::LANGUAGE;

use crate::{
    ParserError, ParserErrorKind, RawNodePath, TransferInstruction, TransferInstructionKind,
};

#[derive(Clone, Debug)]
pub enum TraversalTask {
    Expression(RawNodePath),
    Register(TransferInstruction),
}

pub struct Preparer {
    pub tree_builder: TreeBuilder,
    pub ts_parser: TSParser,
    pub raw_tree: TSTree,
    pub started: bool,
    pub completed: bool,
    pub tasks: Vec<TraversalTask>,
    pub instructions: Vec<TransferInstruction>,
}

impl Preparer {
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
            started: false,
            completed: false,
            tasks: vec![],
            instructions: vec![],
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

    /// Collect all top-level instructions into tasks
    fn start_preparation(&mut self) -> Result<(), ParserError> {
        if self.started {
            return Ok(());
        }

        let root = self.raw_tree.root_node();
        if root.has_error() || root.kind() != "source_file" {
            return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
        }

        if root.named_child_count() != 1
            || root
                .named_child(0)
                .is_none_or(|child| child.kind() != "multi_expression")
        {
            return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
        }

        let multi_expression = root.named_child(0).unwrap();
        let multi_expression_path = RawNodePath::root().with_child(0);
        let mut expression_count = 0;
        let mut tasks = Vec::new();

        for index in (0..multi_expression.named_child_count()).rev() {
            let child = multi_expression.named_child(index as u32).unwrap();
            if child.kind() == "expression_sep" {
                continue;
            }

            tasks.push(TraversalTask::Expression(
                multi_expression_path.with_child(index as u32),
            ));
            expression_count += 1;
        }

        if expression_count == 0 {
            return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
        }

        self.tasks = tasks;
        self.started = true;
        Ok(())
    }

    pub fn prepare_next(&mut self) -> Result<Option<usize>, ParserError> {
        if self.completed {
            return Ok(None);
        }

        self.start_preparation()?;

        while let Some(task) = self.tasks.pop() {
            let path = match task {
                TraversalTask::Register(instruction) => {
                    let instruction_index = self.instructions.len();
                    self.instructions.push(instruction);
                    return Ok(Some(instruction_index));
                }
                TraversalTask::Expression(path) => path,
            };

            let node = path
                .of_tree(&self.raw_tree)
                .ok_or_else(|| ParserError::new(None, ParserErrorKind::InvalidSource))?;

            match node.kind() {
                "identifier" => {
                    let instruction_index = self.instructions.len();
                    self.instructions.push(TransferInstruction::new(
                        TransferInstructionKind::IdentifierExpr,
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "unquoted_string_literal" => {
                    let instruction_index = self.instructions.len();
                    self.instructions.push(TransferInstruction::new(
                        TransferInstructionKind::StringLiteral(StringLiteralKind::Unquoted),
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "squoted_string_literal" => {
                    let instruction_index = self.instructions.len();
                    self.instructions.push(TransferInstruction::new(
                        TransferInstructionKind::StringLiteral(StringLiteralKind::Squoted),
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "dquoted_string_literal" => {
                    let instruction_index = self.instructions.len();
                    self.instructions.push(TransferInstruction::new(
                        TransferInstructionKind::StringLiteral(StringLiteralKind::Dquoted),
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "parenthesis_expression" => {
                    if node.named_child_count() != 1 {
                        return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                    }

                    self.tasks
                        .push(TraversalTask::Expression(path.with_child(0)));
                }
                "binary_expression" => {
                    let add_expression = node.named_child(0);
                    if node.named_child_count() != 1
                        || add_expression.is_none_or(|child| child.kind() != "add_expression")
                    {
                        return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                    }

                    let add_expression = add_expression.unwrap();
                    if add_expression.named_child_count() != 2 {
                        return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                    }

                    let add_path = path.with_child(0);
                    self.tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::Binary(BinaryOp::Add),
                            node.byte_range(),
                        )));
                    self.tasks
                        .push(TraversalTask::Expression(add_path.with_child(1)));
                    self.tasks
                        .push(TraversalTask::Expression(add_path.with_child(0)));
                }
                "function_call" => {
                    let child_count = node.named_child_count();
                    if child_count < 1
                        || node
                            .named_child(0)
                            .is_none_or(|child| child.kind() != "identifier")
                    {
                        return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                    }

                    self.tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::FunctionCall {
                                arg_count: child_count - 1,
                            },
                            node.byte_range(),
                        )));

                    for index in (1..child_count).rev() {
                        self.tasks
                            .push(TraversalTask::Expression(path.with_child(index as u32)));
                    }

                    let identifier = node.named_child(0).unwrap();
                    self.tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::IdentifierPrim,
                            identifier.byte_range(),
                        )));
                }
                _ => {
                    return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                }
            }
        }

        self.completed = true;
        Ok(None)
    }

    pub fn prepare(&mut self) -> Result<usize, ParserError> {
        let mut last_index: usize = 0;
        while let Some(index) = self.prepare_next()? {
            last_index = index;
        }
        Ok(last_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_no_args() {
        let mut preparer = Preparer::from_source("$f()").unwrap();
        preparer.prepare().unwrap();
        assert_eq!(preparer.instructions.len(), 2);
        let expected_instructions = [
            TransferInstructionKind::IdentifierPrim,
            TransferInstructionKind::FunctionCall { arg_count: 0 },
        ];
        let predicted_instructions = preparer
            .instructions
            .iter()
            .map(|i| i.kind.clone())
            .collect::<Vec<TransferInstructionKind>>();
        assert_eq!(predicted_instructions, expected_instructions);
    }

    #[test]
    fn test_prepare_transfer_next() {
        let mut preparer = Preparer::from_source("$f(a + b)").unwrap();

        // make sure only node gets prepared per cycle
        let mut nodes_prepared = 0;
        while let Some(index) = preparer.prepare_next().unwrap() {
            assert_eq!(nodes_prepared, index);
            assert_eq!(index, preparer.instructions.len() - 1);
            nodes_prepared += 1;
        }

        assert_eq!(preparer.instructions.len(), 5);
        let expected_instructions = [
            TransferInstructionKind::IdentifierPrim,
            TransferInstructionKind::StringLiteral(StringLiteralKind::Unquoted),
            TransferInstructionKind::StringLiteral(StringLiteralKind::Unquoted),
            TransferInstructionKind::Binary(BinaryOp::Add),
            TransferInstructionKind::FunctionCall { arg_count: 1 },
        ];
        let predicted_instructions = preparer
            .instructions
            .iter()
            .map(|i| i.kind.clone())
            .collect::<Vec<TransferInstructionKind>>();
        assert_eq!(predicted_instructions, expected_instructions);
    }
}
