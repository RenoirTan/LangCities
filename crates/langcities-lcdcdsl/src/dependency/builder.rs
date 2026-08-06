use std::collections::HashSet;

use crate::{
    dependency::{Dependency, DependencyKind},
    error::{DslError, DslErrorKind},
    node::{NodeId, NodeKind},
    tree::{TraversalKind, Tree, TreeTraverser},
};

pub struct DependencyBuilder<'t> {
    traverser: TreeTraverser<'t>,
    seen: HashSet<Dependency>,
    node_ids: Vec<NodeId>,
}

impl<'t> DependencyBuilder<'t> {
    pub fn new(tree: &'t Tree) -> Result<Self, DslError> {
        let start_id = tree.root_node_id.ok_or_else(|| {
            DslError::new(
                Some("No root node found".into()),
                DslErrorKind::NodeNotFound,
            )
        })?;
        let traverser = TreeTraverser::new(tree, start_id, TraversalKind::Preorder)?;
        Ok(Self {
            traverser,
            seen: HashSet::new(),
            node_ids: vec![],
        })
    }

    pub fn find_next(&mut self) -> Result<Option<&Dependency>, DslError> {
        while let Some(node_id) = self.traverser.find_next()? {
            self.node_ids.push(node_id);

            let node = self
                .traverser
                .tree
                .arena
                .get(&node_id)
                .ok_or_else(|| DslError::node_not_found(node_id))?;

            match &node.node {
                NodeKind::IdentifierPrim(_) => {
                    // check if being used as a function identifier
                    if self.node_ids.len() < 2 {
                        continue;
                    }
                    let penultimate_node_id = self.node_ids[self.node_ids.len() - 2];
                    let penultimate_node = self
                        .traverser
                        .tree
                        .arena
                        .get(&penultimate_node_id)
                        .ok_or_else(|| DslError::node_not_found(penultimate_node_id))?;
                    if let NodeKind::FunctionCallExpr(_) = &penultimate_node.node {
                        let dependency = Dependency::new(
                            node.context.raw(&self.traverser.tree),
                            DependencyKind::Func,
                        );
                        if !self.seen.contains(&dependency) {
                            self.seen.insert(dependency.clone());
                            return Ok(self.seen.get(&dependency));
                        }
                    }
                }
                NodeKind::IdentifierExpr(_) => {
                    let dependency = Dependency::new(
                        node.context.raw(&self.traverser.tree),
                        DependencyKind::Var,
                    );
                    if !self.seen.contains(&dependency) {
                        self.seen.insert(dependency.clone());
                        return Ok(self.seen.get(&dependency));
                    }
                }
                _ => continue,
            }
        }

        Ok(None)
    }

    pub fn find(&mut self) -> Result<&HashSet<Dependency>, DslError> {
        while let Some(_) = self.find_next()? {}
        Ok(&self.seen)
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::tests::create_tree;

    use super::*;

    #[test]
    fn test_dependency_stream() {
        let tree = create_tree();
        let mut builder = DependencyBuilder::new(&tree).unwrap();
        let dependencies = builder.find().unwrap();
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&Dependency::new("$f", DependencyKind::Func)));
        assert!(dependencies.contains(&Dependency::new("$g", DependencyKind::Func)));
    }
}
