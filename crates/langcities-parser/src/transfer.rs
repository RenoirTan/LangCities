use std::ops::Range;

use langcities_dsl::node::{
    BinaryExpr, BinaryOp, FunctionCallExpr, IdentifierExpr, IdentifierPrim, Node, NodeContext,
    NodeId, NodeKind, StringLiteralExpr, StringLiteralKind,
};

use crate::{ParserError, ParserErrorKind, Preparer};

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

pub struct Transferer {
    pub preparer: Preparer,
    pub instruction_index: usize,
    pub postorder_ids: Vec<usize>,
}

impl Transferer {
    pub fn new(preparer: impl Into<Preparer>) -> Self {
        let preparer = preparer.into();
        Self {
            preparer,
            instruction_index: 0,
            postorder_ids: vec![],
        }
    }

    pub fn transfer(&mut self) -> Result<(), ParserError> {
        while self.transfer_next()?.is_some() {}
        Ok(())
    }

    // Read instructions like RPN (because postorder traversal) and combine into native DSL
    // implementation
    pub fn transfer_next(&mut self) -> Result<Option<NodeId>, ParserError> {
        // Check if anymore instructions, otherwise push the preparer for more
        if self.instruction_index == self.preparer.instructions.len()
            && self.preparer.prepare_next()?.is_none()
        {
            return Ok(None);
        }

        let instruction = self.preparer.instructions[self.instruction_index].clone();
        let TransferInstruction { kind, context } = instruction;

        let node = match kind {
            TransferInstructionKind::IdentifierExpr => {
                NodeKind::IdentifierExpr(IdentifierExpr::default())
            }
            TransferInstructionKind::IdentifierPrim => NodeKind::IdentifierPrim(IdentifierPrim),
            TransferInstructionKind::StringLiteral(kind) => {
                NodeKind::StringLiteralExpr(StringLiteralExpr::new(kind))
            }
            TransferInstructionKind::Binary(op) => {
                if self.postorder_ids.len() < 2 {
                    return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                }

                let child_ids = self.postorder_ids.split_off(self.postorder_ids.len() - 2);
                NodeKind::BinaryExpr(BinaryExpr::new(op, child_ids[0], child_ids[1]))
            }
            TransferInstructionKind::FunctionCall { arg_count } => {
                let child_count = arg_count + 1;
                if self.postorder_ids.len() < child_count {
                    return Err(ParserError::new(None, ParserErrorKind::InvalidSource));
                }

                let child_ids = self
                    .postorder_ids
                    .split_off(self.postorder_ids.len() - child_count);
                NodeKind::FunctionCallExpr(FunctionCallExpr::new(
                    child_ids[0],
                    child_ids[1..].to_vec(),
                ))
            }
        };

        let node_id = self.register_node(node, context.span);
        self.postorder_ids.push(node_id);
        self.instruction_index += 1;
        Ok(Some(node_id))
    }

    fn register_node(&mut self, node: NodeKind, span: Range<usize>) -> NodeId {
        let node_id = self.preparer.tree_builder.get_next_node_id();
        let _ = self
            .preparer
            .tree_builder
            .register_node(Node::new(node, NodeContext { node_id, span }));
        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_next() {
        let preparer = Preparer::from_source("$f(a + b)").unwrap();
        let mut ids = Vec::new();
        let mut transferer = Transferer::new(preparer);

        loop {
            let previous_len = transferer.preparer.tree_builder.tree.arena.len();
            let Some(node_id) = transferer.transfer_next().unwrap() else {
                assert_eq!(
                    transferer.preparer.tree_builder.tree.arena.len(),
                    previous_len
                );
                break;
            };

            assert_eq!(
                transferer.preparer.tree_builder.tree.arena.len(),
                previous_len + 1
            );
            assert!(
                transferer
                    .preparer
                    .tree_builder
                    .tree
                    .arena
                    .contains_key(&node_id)
            );
            ids.push(node_id);
        }

        assert_eq!(ids, [0, 1, 2, 3, 4]);

        let NodeKind::BinaryExpr(binary) =
            &transferer.preparer.tree_builder.tree.arena[&ids[3]].node
        else {
            panic!("expected binary expression");
        };
        assert_eq!(binary.left_id, 1);
        assert_eq!(binary.right_id, 2);

        let NodeKind::FunctionCallExpr(func_call) =
            &transferer.preparer.tree_builder.tree.arena[&4].node
        else {
            panic!("expected function call");
        };
        assert_eq!(func_call.identifier_id, ids[0]);
        assert_eq!(func_call.arg_ids, [3]);
    }

    #[test]
    fn test_transfer_consumes_prepared_instructions() {
        let mut preparer = Preparer::from_source("$f(a + b)").unwrap();

        while preparer.prepare_next().unwrap().is_some() {}
        assert!(preparer.tree_builder.tree.arena.is_empty());

        let mut transferer = Transferer::new(preparer);

        transferer.transfer().unwrap();

        assert_eq!(transferer.preparer.tree_builder.tree.arena.len(), 5);
        assert_eq!(transferer.transfer_next().unwrap(), None);
    }

    #[test]
    fn test_invalid_source_registers_nothing() {
        let preparer = Preparer::from_source("$value +").unwrap();
        let mut transferer = Transferer::new(preparer);

        assert!(transferer.transfer_next().is_err());
        assert!(transferer.preparer.tree_builder.tree.arena.is_empty());
        assert_eq!(transferer.preparer.tree_builder.next_node_id, 0);
    }
}
