use std::ops::Range;

use langcities_dsl::{
    node::{
        BinaryExpression, BinaryOp, Expr, ExpressionNode, FunctionCall, IdentifierExpression,
        IdentifierPrimitive, Node, NodeContext, NodeId, NodeKind, Prim, PrimitiveNode,
        StringLiteral, StringLiteralKind,
    },
    tree::TreeBuilder,
};
use tree_sitter::{Node as TSNode, Parser as TSParser, Tree as TSTree};
use tree_sitter_lcdcdsl::LANGUAGE;

use crate::{ParserError, ParserErrorKind};

#[derive(Clone, Debug, PartialEq, Eq)]
struct TransferInstructionContext {
    span: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TransferInstruction {
    kind: TransferInstructionKind,
    context: TransferInstructionContext,
}

impl TransferInstruction {
    fn new(kind: TransferInstructionKind, span: Range<usize>) -> Self {
        Self {
            kind,
            context: TransferInstructionContext { span },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TransferInstructionKind {
    IdentifierExpression,
    IdentifierPrimitive,
    StringLiteral(StringLiteralKind),
    Binary(BinaryOp),
    FunctionCall { arg_count: usize },
}

#[derive(Clone, Debug)]
struct RawNodePath {
    indices: Vec<u32>,
}

impl RawNodePath {
    fn root() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    fn with_child(&self, index: u32) -> Self {
        let mut indices = self.indices.clone();
        indices.push(index);
        Self { indices }
    }
}

#[derive(Debug)]
enum TraversalTask {
    Expression(RawNodePath),
    Register(TransferInstruction),
}

pub struct Parser {
    pub tree_builder: TreeBuilder,
    pub ts_parser: TSParser,
    pub raw_tree: TSTree,
    transfer_instructions: Vec<TransferInstruction>,
    next_transfer_instruction: usize,
    transfer_values: Vec<NodeId>,
    transfer_tasks: Vec<TraversalTask>,
    transfer_preparation_started: bool,
    transfer_preparation_complete: bool,
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
            transfer_instructions: Vec::new(),
            next_transfer_instruction: 0,
            transfer_values: Vec::new(),
            transfer_tasks: Vec::new(),
            transfer_preparation_started: false,
            transfer_preparation_complete: false,
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
        while self.transfer_next()?.is_some() {}
        Ok(())
    }

    pub fn prepare_transfer_next(&mut self) -> Result<Option<usize>, ParserError> {
        if self.transfer_preparation_complete {
            return Ok(None);
        }

        self.start_transfer_preparation()?;

        while let Some(task) = self.transfer_tasks.pop() {
            let path = match task {
                TraversalTask::Register(instruction) => {
                    let instruction_index = self.transfer_instructions.len();
                    self.transfer_instructions.push(instruction);
                    return Ok(Some(instruction_index));
                }
                TraversalTask::Expression(path) => path,
            };

            let node = Self::node_at_path(&self.raw_tree, &path)
                .ok_or_else(|| ParserError::new(None, ParserErrorKind::InvalidSource))?;

            match node.kind() {
                "identifier" => {
                    let instruction_index = self.transfer_instructions.len();
                    self.transfer_instructions.push(TransferInstruction::new(
                        TransferInstructionKind::IdentifierExpression,
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "unquoted_string_literal" => {
                    let instruction_index = self.transfer_instructions.len();
                    self.transfer_instructions.push(TransferInstruction::new(
                        TransferInstructionKind::StringLiteral(StringLiteralKind::Unquoted),
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "squoted_string_literal" => {
                    let instruction_index = self.transfer_instructions.len();
                    self.transfer_instructions.push(TransferInstruction::new(
                        TransferInstructionKind::StringLiteral(StringLiteralKind::Squoted),
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "dquoted_string_literal" => {
                    let instruction_index = self.transfer_instructions.len();
                    self.transfer_instructions.push(TransferInstruction::new(
                        TransferInstructionKind::StringLiteral(StringLiteralKind::Dquoted),
                        node.byte_range(),
                    ));
                    return Ok(Some(instruction_index));
                }
                "parenthesis_expression" => {
                    if node.named_child_count() != 1 {
                        return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                    }

                    self.transfer_tasks
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
                    self.transfer_tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::Binary(BinaryOp::Add),
                            node.byte_range(),
                        )));
                    self.transfer_tasks
                        .push(TraversalTask::Expression(add_path.with_child(1)));
                    self.transfer_tasks
                        .push(TraversalTask::Expression(add_path.with_child(0)));
                }
                "function_call" => {
                    let child_count = node.named_child_count();
                    if child_count < 2
                        || node
                            .named_child(0)
                            .is_none_or(|child| child.kind() != "identifier")
                    {
                        return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                    }

                    self.transfer_tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::FunctionCall {
                                arg_count: child_count - 1,
                            },
                            node.byte_range(),
                        )));

                    for index in (1..child_count).rev() {
                        self.transfer_tasks
                            .push(TraversalTask::Expression(path.with_child(index as u32)));
                    }

                    let identifier = node.named_child(0).unwrap();
                    self.transfer_tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::IdentifierPrimitive,
                            identifier.byte_range(),
                        )));
                }
                _ => {
                    return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                }
            }
        }

        self.transfer_preparation_complete = true;
        Ok(None)
    }

    pub fn transfer_next(&mut self) -> Result<Option<NodeId>, ParserError> {
        if self.next_transfer_instruction == self.transfer_instructions.len()
            && self.prepare_transfer_next()?.is_none()
        {
            return Ok(None);
        }

        let instruction = self.transfer_instructions[self.next_transfer_instruction].clone();
        let TransferInstruction { kind, context } = instruction;

        let node = match kind {
            TransferInstructionKind::IdentifierExpression => NodeKind::Expr(ExpressionNode {
                expr: Expr::Identifier(IdentifierExpression::from(IdentifierPrimitive)),
            }),
            TransferInstructionKind::IdentifierPrimitive => NodeKind::Prim(PrimitiveNode {
                prim: Prim::Identifier(IdentifierPrimitive),
            }),
            TransferInstructionKind::StringLiteral(kind) => NodeKind::Expr(ExpressionNode {
                expr: Expr::StringLiteral(StringLiteral::new(kind)),
            }),
            TransferInstructionKind::Binary(op) => {
                if self.transfer_values.len() < 2 {
                    return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                }

                let child_ids = self
                    .transfer_values
                    .split_off(self.transfer_values.len() - 2);
                NodeKind::Expr(ExpressionNode {
                    expr: Expr::Binary(BinaryExpression::new(op, child_ids[0], child_ids[1])),
                })
            }
            TransferInstructionKind::FunctionCall { arg_count } => {
                let child_count = arg_count + 1;
                if self.transfer_values.len() < child_count {
                    return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                }

                let child_ids = self
                    .transfer_values
                    .split_off(self.transfer_values.len() - child_count);
                NodeKind::Expr(ExpressionNode {
                    expr: Expr::FunctionCall(FunctionCall::new(
                        child_ids[0],
                        child_ids[1..].to_vec(),
                    )),
                })
            }
        };

        let node_id = self.register_node(node, context.span);
        self.transfer_values.push(node_id);
        self.next_transfer_instruction += 1;
        Ok(Some(node_id))
    }

    /// Collect all top-level instructions into tasks
    fn start_transfer_preparation(&mut self) -> Result<(), ParserError> {
        if self.transfer_preparation_started {
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

        self.transfer_tasks = tasks;
        self.transfer_preparation_started = true;
        Ok(())
    }

    fn register_node(&mut self, node: NodeKind, span: Range<usize>) -> NodeId {
        let node_id = self.tree_builder.get_next_node_id();
        let _ = self
            .tree_builder
            .register_node(Node::new(node, NodeContext { node_id, span }));
        node_id
    }

    fn node_at_path<'tree>(tree: &'tree TSTree, path: &RawNodePath) -> Option<TSNode<'tree>> {
        let mut node = tree.root_node();
        for index in &path.indices {
            node = node.named_child(*index)?;
        }
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_transfer_next() {
        let mut parser = Parser::from_source("$f(a + b)").unwrap();

        // make sure only node gets prepared per cycle
        let mut nodes_prepared = 0;
        while let Some(index) = parser.prepare_transfer_next().unwrap() {
            assert_eq!(nodes_prepared, index);
            assert_eq!(index, parser.transfer_instructions.len() - 1);
            nodes_prepared += 1;
        }

        assert_eq!(parser.transfer_instructions.len(), 5);
        let expected_instructions = [
            TransferInstructionKind::IdentifierPrimitive,
            TransferInstructionKind::StringLiteral(StringLiteralKind::Unquoted),
            TransferInstructionKind::StringLiteral(StringLiteralKind::Unquoted),
            TransferInstructionKind::Binary(BinaryOp::Add),
            TransferInstructionKind::FunctionCall { arg_count: 1 },
        ];
        let predicted_instructions = parser
            .transfer_instructions
            .iter()
            .map(|i| i.kind.clone())
            .collect::<Vec<TransferInstructionKind>>();
        assert_eq!(predicted_instructions, expected_instructions);
    }

    #[test]
    fn test_transfer_next() {
        let mut parser = Parser::from_source("$f(a + b)").unwrap();
        let mut ids = Vec::new();

        loop {
            let previous_len = parser.tree_builder.tree.arena.len();
            let Some(node_id) = parser.transfer_next().unwrap() else {
                assert_eq!(parser.tree_builder.tree.arena.len(), previous_len);
                break;
            };

            assert_eq!(parser.tree_builder.tree.arena.len(), previous_len + 1);
            assert!(parser.tree_builder.tree.arena.contains_key(&node_id));
            ids.push(node_id);
        }

        assert_eq!(ids, [0, 1, 2, 3, 4]);

        let NodeKind::Expr(ExpressionNode {
            expr: Expr::Binary(binary),
        }) = &parser.tree_builder.tree.arena[&ids[3]].node
        else {
            panic!("expected binary expression");
        };
        assert_eq!(binary.left_id, 1);
        assert_eq!(binary.right_id, 2);

        let NodeKind::Expr(ExpressionNode {
            expr: Expr::FunctionCall(function_call),
        }) = &parser.tree_builder.tree.arena[&4].node
        else {
            panic!("expected function call");
        };
        assert_eq!(function_call.identifier, ids[0]);
        assert_eq!(function_call.args, [3]);
    }

    #[test]
    fn test_transfer_consumes_prepared_instructions() {
        let mut parser = Parser::from_source("$f(a + b)").unwrap();

        while parser.prepare_transfer_next().unwrap().is_some() {}
        assert!(parser.tree_builder.tree.arena.is_empty());

        parser.transfer().unwrap();

        assert_eq!(parser.tree_builder.tree.arena.len(), 5);
        assert_eq!(parser.transfer_next().unwrap(), None);
    }

    #[test]
    fn test_invalid_source_registers_nothing() {
        let mut parser = Parser::from_source("$value +").unwrap();

        assert!(parser.transfer_next().is_err());
        assert!(parser.tree_builder.tree.arena.is_empty());
        assert_eq!(parser.tree_builder.next_node_id, 0);
    }
}
