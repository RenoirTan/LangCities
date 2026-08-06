use crate::{
    error::{DslError, DslErrorKind},
    node::{NodeId, NodeKind},
    tree::Tree,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TraversalKind {
    Bfs,
    Preorder,
    Postorder,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct TraversalTask {
    pub node_id: NodeId,
    pub initial: bool,
}

impl TraversalTask {
    fn new<N, I>(node_id: N, initial: I) -> Self
    where
        N: Into<NodeId>,
        I: Into<bool>,
    {
        let (node_id, initial) = (node_id.into(), initial.into());
        Self { node_id, initial }
    }

    fn new_initial(node_id: impl Into<NodeId>) -> Self {
        Self::new(node_id, true)
    }
}

#[derive(Clone, Debug)]
pub struct TreeTraverser<'t> {
    pub tree: &'t Tree,
    tasks: Vec<TraversalTask>,
    kind: TraversalKind,
}

impl<'t> TreeTraverser<'t> {
    pub fn new<S, K>(tree: &'t Tree, start_id: S, kind: K) -> Result<Self, DslError>
    where
        S: Into<NodeId>,
        K: Into<TraversalKind>,
    {
        let (start_id, kind) = (start_id.into(), kind.into());
        tree.arena.get(&start_id).ok_or_else(|| {
            DslError::new(
                Some(format!("Node {} not found", start_id).into()),
                DslErrorKind::NodeNotFound,
            )
        })?;
        Ok(Self {
            tree,
            tasks: vec![TraversalTask::new_initial(start_id)],
            kind,
        })
    }

    fn append_next(&mut self, mut tasks: Vec<TraversalTask>) {
        if matches!(
            self.kind,
            TraversalKind::Preorder | TraversalKind::Postorder
        ) {
            tasks.reverse();
        }
        self.tasks.append(&mut tasks);
    }

    fn pop_next(&mut self) -> Option<TraversalTask> {
        match self.kind {
            TraversalKind::Bfs => {
                if self.tasks.is_empty() {
                    None
                } else {
                    Some(self.tasks.remove(0))
                }
            }
            _ => self.tasks.pop(),
        }
    }

    pub fn find_next(&mut self) -> Result<Option<NodeId>, DslError> {
        while let Some(task) = self.pop_next() {
            let TraversalTask { node_id, initial } = task;
            let node = self.tree.arena.get(&node_id).ok_or_else(|| {
                DslError::new(
                    Some(format!("Node {} not found", node_id).into()),
                    DslErrorKind::NodeNotFound,
                )
            })?;
            match &node.node {
                NodeKind::IdentifierExpr(_)
                | NodeKind::IdentifierPrim(_)
                | NodeKind::StringLiteralExpr(_) => {
                    return Ok(Some(node_id));
                }
                NodeKind::BinaryExpr(b) => {
                    if initial {
                        let mut tasks = vec![
                            TraversalTask::new_initial(b.left_id),
                            TraversalTask::new_initial(b.right_id),
                        ];
                        if matches!(self.kind, TraversalKind::Postorder) {
                            tasks.push(TraversalTask::new(node_id, false));
                            self.append_next(tasks);
                        } else {
                            self.append_next(tasks);
                            return Ok(Some(node_id));
                        }
                    } else {
                        return Ok(Some(node_id));
                    }
                }
                NodeKind::FunctionCallExpr(f) => {
                    if initial {
                        let mut ids = vec![f.identifier_id];
                        ids.extend_from_slice(&f.arg_ids);
                        let mut tasks = ids
                            .into_iter()
                            .map(TraversalTask::new_initial)
                            .collect::<Vec<_>>();
                        if matches!(self.kind, TraversalKind::Postorder) {
                            tasks.push(TraversalTask::new(node_id, false));
                            self.append_next(tasks);
                        } else {
                            self.append_next(tasks);
                            return Ok(Some(node_id));
                        }
                    } else {
                        return Ok(Some(node_id));
                    }
                }
                _ => {
                    return Err(DslError::new(
                        Some(
                            format!("Unimplemented node {} of kind: {:?}", node_id, node.node,)
                                .into(),
                        ),
                        DslErrorKind::NodeNotFound,
                    ));
                }
            }
        }
        Ok(None)
    }

    pub fn try_collect(&mut self) -> Result<Vec<NodeId>, DslError> {
        let mut node_ids = vec![];
        while let Some(node_id) = self.find_next()? {
            node_ids.push(node_id);
        }
        Ok(node_ids)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::tests::{create_tree_0, create_tree_1};

    use super::*;

    #[test]
    fn test_bfs_0() {
        let tree = create_tree_0();
        let mut traverser =
            TreeTraverser::new(&tree, tree.root_node_id.unwrap(), TraversalKind::Bfs).unwrap();
        let path = traverser.try_collect().unwrap();
        assert_eq!(path, [7, 0, 6, 4, 5, 1, 2, 3]);
    }

    #[test]
    fn test_preorder_0() {
        let tree = create_tree_0();
        let mut traverser =
            TreeTraverser::new(&tree, tree.root_node_id.unwrap(), TraversalKind::Preorder).unwrap();
        let path = traverser.try_collect().unwrap();
        assert_eq!(path, [7, 0, 6, 4, 1, 2, 3, 5]);
    }

    #[test]
    fn test_postorder_0() {
        let tree = create_tree_0();
        let mut traverser =
            TreeTraverser::new(&tree, tree.root_node_id.unwrap(), TraversalKind::Postorder)
                .unwrap();
        let path = traverser.try_collect().unwrap();
        assert_eq!(path, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_bfs_1() {
        let tree = create_tree_1();
        let mut traverser =
            TreeTraverser::new(&tree, tree.root_node_id.unwrap(), TraversalKind::Bfs).unwrap();
        let path = traverser.try_collect().unwrap();
        assert_eq!(path, [9, 5, 8, 1, 4, 6, 7, 2, 3]);
    }

    #[test]
    fn test_preorder_1() {
        let tree = create_tree_1();
        let mut traverser =
            TreeTraverser::new(&tree, tree.root_node_id.unwrap(), TraversalKind::Preorder).unwrap();
        let path = traverser.try_collect().unwrap();
        assert_eq!(path, [9, 5, 1, 4, 2, 3, 8, 6, 7]);
    }

    #[test]
    fn test_postorder_1() {
        let tree = create_tree_1();
        let mut traverser =
            TreeTraverser::new(&tree, tree.root_node_id.unwrap(), TraversalKind::Postorder)
                .unwrap();
        let path = traverser.try_collect().unwrap();
        assert_eq!(path, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
