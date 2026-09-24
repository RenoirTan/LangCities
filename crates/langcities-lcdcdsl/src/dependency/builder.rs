use crate::{
    dependency::{DepData, Dependency},
    error::{DslError, DslErrorTrait},
    node::{NodeId, NodeKind},
    tree::{TraversalKind, Tree, TreeTraverser},
};

pub struct DependencyBuilder<'t> {
    traverser: TreeTraverser<'t>,
    list: Vec<Dependency>,
    node_ids: Vec<NodeId>,
}

impl<'t> DependencyBuilder<'t> {
    pub fn new(tree: &'t Tree) -> Result<Self, DslError> {
        let start_id = tree
            .root_node_id
            .ok_or_else(|| DslError::node_not_found("tree has no root node"))?;
        let traverser = TreeTraverser::new(tree, start_id, TraversalKind::Preorder)?;
        Ok(Self {
            traverser,
            list: vec![],
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
                .ok_or_else(|| DslError::node_not_found_of(node_id))?;

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
                        .ok_or_else(|| DslError::node_not_found_of(penultimate_node_id))?;
                    if let NodeKind::FunctionCallExpr(_) = &penultimate_node.node {
                        let dependency = Dependency::new(
                            node.context.raw(&self.traverser.tree),
                            DepData::new(node_id),
                        );
                        self.list.push(dependency.clone());
                        return Ok(self.list.last());
                    }
                }
                NodeKind::IdentifierExpr(_) => {
                    let dependency = Dependency::new(
                        node.context.raw(&self.traverser.tree),
                        DepData::new(node_id),
                    );
                    self.list.push(dependency.clone());
                    return Ok(self.list.last());
                }
                _ => continue,
            }
        }

        Ok(None)
    }

    pub fn find(&mut self) -> Result<&Vec<Dependency>, DslError> {
        while let Some(_) = self.find_next()? {}
        println!("{:#?}", self.list);
        Ok(&self.list)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dependency::DepMap,
        tests::{create_tree_0, create_tree_1},
    };

    use super::*;

    #[test]
    fn test_dependency_stream_0() {
        let tree = create_tree_0();
        let mut builder = DependencyBuilder::new(&tree).unwrap();
        let dep_map: DepMap = builder.find().unwrap().clone().into_iter().collect();
        assert_eq!(dep_map.len(), 2);
        let f = Dependency::new("$f", DepData::new(0 as NodeId));
        let g = Dependency::new("$g", DepData::new(1 as NodeId));
        assert!(dep_map.has_dep(&f));
        assert!(dep_map.has_dep(&g));
        assert!(dep_map.has_identifier(&f.identifier));
        assert!(dep_map.has_identifier(&g.identifier));
    }

    #[test]
    fn test_dependency_stream_1() {
        let tree = create_tree_1();
        let mut builder = DependencyBuilder::new(&tree).unwrap();
        let dep_map: DepMap = builder.find().unwrap().clone().into_iter().collect();
        assert_eq!(dep_map.len(), 4);
        let otmt_0 = Dependency::new("$mt.sc.ot_mt", DepData::new(0 as NodeId));
        let pdot = Dependency::new("$ot.sc.pd_ot", DepData::new(1 as NodeId));
        let otmt_1 = Dependency::new("$mt.sc.ot_mt", DepData::new(7 as NodeId));
        let identifier = Dependency::new("$identifier", DepData::new(8 as NodeId));
        assert_eq!(otmt_0.identifier, otmt_1.identifier);
        assert!(dep_map.has_dep(&otmt_0));
        assert!(dep_map.has_dep(&pdot));
        assert!(dep_map.has_dep(&otmt_1));
        assert!(dep_map.has_dep(&identifier));
        assert!(dep_map.has_identifier(&otmt_0.identifier));
        assert!(dep_map.has_identifier(&pdot.identifier));
        assert!(dep_map.has_identifier(&otmt_1.identifier));
        assert!(dep_map.has_identifier(&identifier.identifier));
    }
}
