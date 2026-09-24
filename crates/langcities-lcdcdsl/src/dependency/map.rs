use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::dependency::{DepData, Dependency};

type Inner = HashMap<String, HashSet<DepData>>;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DepMap {
    inner: Inner,
}

impl DepMap {
    pub fn new(inner: impl Into<Inner>) -> Self {
        let inner = inner.into();
        Self { inner }
    }

    pub fn len(&self) -> usize {
        self.inner.values().map(|s| s.len()).sum()
    }

    pub fn identifiers(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    pub fn has_identifier<Q>(&self, identifier: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.contains_key(identifier)
    }

    pub fn has_dep(&self, dep: &Dependency) -> bool {
        self.get_identdepdata(dep).is_some()
    }

    pub fn get_identdepdata(&self, dep: &Dependency) -> Option<(&String, &DepData)> {
        let (identifier, datas) = self.inner.get_key_value(&dep.identifier)?;
        let data = datas.get(&dep.data)?;
        Some((identifier, data))
    }

    pub fn get_cloned_dep(&self, dep: &Dependency) -> Option<Dependency> {
        let (identifier, data) = self.get_identdepdata(dep)?;
        Some(Dependency::new(identifier.clone(), data.clone()))
    }

    pub fn get_depdatas<Q>(&self, identifier: &Q) -> Option<&HashSet<DepData>>
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get(identifier)
    }

    pub fn iter_deps_of<Q>(&self, identifier: &Q) -> impl Iterator<Item = Dependency>
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner
            .get_key_value(identifier)
            .into_iter()
            .map(|(i, s)| s.iter().map(|d| Dependency::new(i.clone(), d.clone())))
            .flatten()
    }

    pub fn iter_depdatas_of<Q>(&self, identifier: &Q) -> impl Iterator<Item = &DepData>
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner
            .get(identifier)
            .into_iter()
            .map(|s| s.iter())
            .flatten()
    }

    pub fn iter_deps(&self) -> impl Iterator<Item = Dependency> {
        self.inner
            .iter()
            .map(|(i, s)| s.iter().map(|d| Dependency::new(i.clone(), d.clone())))
            .flatten()
    }

    pub fn into_iter_deps(self) -> impl Iterator<Item = Dependency> {
        self.inner
            .into_iter()
            .map(|(i, s)| s.into_iter().map(move |d| Dependency::new(i.clone(), d)))
            .flatten()
    }

    pub fn insert(&mut self, dep: Dependency) -> bool {
        self.inner
            .entry(dep.identifier)
            .or_insert_with(HashSet::new)
            .insert(dep.data)
    }
}

impl FromIterator<Dependency> for DepMap {
    fn from_iter<T: IntoIterator<Item = Dependency>>(iter: T) -> Self {
        iter.into_iter().fold(Self::default(), |mut m, d| {
            m.insert(d);
            m
        })
    }
}
