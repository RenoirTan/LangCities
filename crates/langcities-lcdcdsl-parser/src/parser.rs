use std::ops::Range;

use langcities_lcdcdsl::{
    node::{
        BinaryExpr, BinaryOp, FunctionCallExpr, IdentifierExpr, IdentifierPrim, Node, NodeContext,
        NodeId, NodeKind, StringLiteralExpr, StringLiteralKind,
    },
    tree::TreeBuilder,
};
use tree_sitter::{Parser as TSParser, Tree as TSTree};
use tree_sitter_lcdcdsl::LANGUAGE;

use crate::{ParserError, ParserErrorKind, RawNodePath};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferInstructionContext {
    pub span: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferInstruction {
    pub kind: TransferInstructionKind,
    pub context: TransferInstructionContext,
}

impl TransferInstruction {
    pub fn new(kind: TransferInstructionKind, span: Range<usize>) -> Self {
        Self {
            kind,
            context: TransferInstructionContext { span },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferInstructionKind {
    IdentifierExpr,
    IdentifierPrim,
    StringLiteral(StringLiteralKind),
    Binary(BinaryOp),
    FunctionCall { arg_count: usize },
}

#[derive(Clone, Debug)]
pub enum TraversalTask {
    Expression(RawNodePath),
    Register(TransferInstruction),
}

pub struct Parser {
    pub tree_builder: TreeBuilder,
    pub ts_parser: TSParser,
    pub raw_tree: TSTree,
    /// Status variable to prevent devs from messing with the order of execution
    started: bool,
    /// Stack of tasks. Basically DFS as we traverse down raw_tree.
    tasks: Vec<TraversalTask>,
    /// Stack of node ids.
    /// Stores the ids of nodes already built, with the deepest nodes nearer to
    /// the exit (?) of the stack.
    /// Before a higher-level node is registered, it pops some of its child node ids from this stack.
    node_ids_stack: Vec<NodeId>,
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
            started: false,
            tasks: vec![],
            node_ids_stack: vec![],
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

    /// Collect the root expression into tasks
    pub fn prepare(&mut self) -> Result<(), ParserError> {
        if self.started {
            return Ok(());
        }

        let root = self.raw_tree.root_node();
        if root.has_error() || root.kind() != "source_file" || root.named_child_count() != 1 {
            return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
        }

        self.tasks = vec![TraversalTask::Expression(
            RawNodePath::root().with_child(0),
        )];
        self.started = true;
        Ok(())
    }

    /// Visit the next node primogeniture (DFS) style
    pub fn transfer_next(&mut self) -> Result<Option<NodeId>, ParserError> {
        self.prepare()?;

        while let Some(task) = self.tasks.pop() {
            let path = match task {
                // If register the node to the tree,
                // Pop children from node_ids_stack
                // Push your own node id to the stack
                // Return node id as the result of one successful iteration
                TraversalTask::Register(instruction) => {
                    let node_kind = match instruction.kind {
                        TransferInstructionKind::IdentifierExpr => {
                            NodeKind::IdentifierExpr(IdentifierExpr::default())
                        }
                        TransferInstructionKind::IdentifierPrim => {
                            NodeKind::IdentifierPrim(IdentifierPrim)
                        }
                        TransferInstructionKind::StringLiteral(kind) => {
                            NodeKind::StringLiteralExpr(StringLiteralExpr::new(kind))
                        }
                        TransferInstructionKind::Binary(op) => {
                            if self.node_ids_stack.len() < 2 {
                                return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                            }

                            let child_ids =
                                self.node_ids_stack.split_off(self.node_ids_stack.len() - 2);
                            NodeKind::BinaryExpr(BinaryExpr::new(op, child_ids[0], child_ids[1]))
                        }
                        TransferInstructionKind::FunctionCall { arg_count } => {
                            let child_count = arg_count + 1;
                            if self.node_ids_stack.len() < child_count {
                                return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                            }

                            let child_ids = self
                                .node_ids_stack
                                .split_off(self.node_ids_stack.len() - child_count);
                            NodeKind::FunctionCallExpr(FunctionCallExpr::new(
                                child_ids[0],
                                child_ids[1..].to_vec(),
                            ))
                        }
                    };
                    let node_id = self.register_node(node_kind, instruction.context.span);
                    // Add to stack so that higher-level expression can use it
                    self.node_ids_stack.push(node_id);
                    if self.tasks.is_empty() {
                        self.tree_builder.tree.root_node_id = Some(node_id);
                    }
                    return Ok(Some(node_id));
                }
                TraversalTask::Expression(path) => path,
            };

            let node = path
                .of_tree(&self.raw_tree)
                .ok_or_else(|| ParserError::new(None, ParserErrorKind::InvalidSource))?;

            match node.kind() {
                "identifier" => {
                    self.tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::IdentifierExpr,
                            node.byte_range(),
                        )));
                }
                "unquoted_string_literal" => {
                    self.tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::StringLiteral(StringLiteralKind::Unquoted),
                            node.byte_range(),
                        )));
                }
                "squoted_string_literal" => {
                    self.tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::StringLiteral(StringLiteralKind::Squoted),
                            node.byte_range(),
                        )));
                }
                "dquoted_string_literal" => {
                    self.tasks
                        .push(TraversalTask::Register(TransferInstruction::new(
                            TransferInstructionKind::StringLiteral(StringLiteralKind::Dquoted),
                            node.byte_range(),
                        )));
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

        Ok(None)
    }

    pub fn transfer(&mut self) -> Result<NodeId, ParserError> {
        let mut last_index: usize = 0;
        while let Some(index) = self.transfer_next()? {
            last_index = index;
        }
        Ok(last_index)
    }

    pub fn register_node(&mut self, node: NodeKind, span: Range<usize>) -> NodeId {
        let node_id = self.tree_builder.get_next_node_id();
        let _ = self
            .tree_builder
            .register_node(Node::new(node, NodeContext { node_id, span }));
        node_id
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn get_ordered_node_kinds(arena: &HashMap<NodeId, Node>) -> Vec<NodeKind> {
        let mut ks = arena
            .iter()
            .map(|n| (*n.0, n.1.node.clone()))
            .collect::<Vec<_>>();
        ks.sort_by_key(|(id, _)| *id);
        ks.into_iter().map(|(_, k)| k).collect::<Vec<_>>()
    }

    #[test]
    fn test_function_no_args() {
        let mut parser = Parser::from_source("$f()").unwrap();
        parser.prepare().unwrap();

        let mut node_ids = vec![];
        while let Some(node_id) = parser.transfer_next().unwrap() {
            node_ids.push(node_id);
        }

        assert_eq!(node_ids, [0, 1]);

        let expected_node_kinds = [
            NodeKind::IdentifierPrim(IdentifierPrim),
            NodeKind::FunctionCallExpr(FunctionCallExpr::new(0 as NodeId, [])),
        ];
        let predicted_node_kinds = get_ordered_node_kinds(&parser.tree_builder.tree.arena);
        assert_eq!(&expected_node_kinds[..], predicted_node_kinds);
        assert_eq!(parser.tree_builder.tree.root_node_id, Some(1));
    }

    #[test]
    fn test_complex_function() {
        let mut parser = Parser::from_source("$mt.sc.ot_mt(a + b, $ot.sc.pd_ot('c'))").unwrap();
        parser.prepare().unwrap();

        let mut node_ids = vec![];
        while let Some(node_id) = parser.transfer_next().unwrap() {
            node_ids.push(node_id);
        }

        assert_eq!(node_ids, (0..8).collect::<Vec<NodeId>>());

        let expected_node_kinds = [
            NodeKind::IdentifierPrim(IdentifierPrim), // $mt.sc.ot_mt
            NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Unquoted)), // a
            NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Unquoted)), // b
            // a + b
            NodeKind::BinaryExpr(BinaryExpr::new(BinaryOp::Add, 1 as NodeId, 2 as NodeId)),
            NodeKind::IdentifierPrim(IdentifierPrim), // $ot.sc.pd_ot
            NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Squoted)), // 'c'
            // $ot.sc.pd_ot('c')
            NodeKind::FunctionCallExpr(FunctionCallExpr::new(4 as NodeId, [5])),
            NodeKind::FunctionCallExpr(FunctionCallExpr::new(0 as NodeId, [3, 6])),
        ];
        let predicted_node_kinds = get_ordered_node_kinds(&parser.tree_builder.tree.arena);
        assert_eq!(&expected_node_kinds[..], predicted_node_kinds);
        assert_eq!(parser.tree_builder.tree.root_node_id, Some(7));
    }

    #[test]
    fn test_invalid_binary_expression() {
        let mut parser = Parser::from_source("$value +").unwrap();
        assert!(parser.prepare().is_err());
        assert!(parser.tree_builder.tree.arena.is_empty());
        assert_eq!(parser.tree_builder.next_node_id, 0);
        assert_eq!(parser.tree_builder.tree.root_node_id, None);
    }
}
