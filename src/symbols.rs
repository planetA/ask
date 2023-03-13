use anyhow::Result;
use clang_ast::SourceRange;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::{collections::HashMap, hash, hash::Hasher};

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Copy, Clone, Serialize, Deserialize)]
pub struct FileHash(u64);

impl FileHash {
    pub fn new<T: hash::Hash>(url: &T) -> Self {
        let mut s = DefaultHasher::new();
        url.hash(&mut s);
        FileHash(s.finish())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Symbol {
    pub name: String,
    pub ranges: Vec<SourceRange>,
    pub children: Vec<SymbolId>,
}

pub trait Symbols: ToString {
    fn add(&mut self, id: SymbolId, symbol: Symbol);
    fn into_vec(&self) -> Vec<SymbolId>;
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SymbolId(String);

impl SymbolId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SymbolMap {
    pub map: HashMap<SymbolId, Symbol>,
}

impl SymbolMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn merge(&mut self, other: SymbolMap) -> &mut Self {
        self.map.extend(other.map);
        self
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&SymbolId, &Symbol)> + 'a {
        self.map.iter()
    }
}

impl Symbols for SymbolMap {
    fn add(&mut self, id: SymbolId, mut symbol: Symbol) {
        if let Some(existing) = self.map.get_mut(&id) {
            assert_eq!(existing.name, symbol.name);
            existing.ranges.append(&mut symbol.ranges);
            existing.children.append(&mut symbol.children);
        } else {
            self.map.insert(id, symbol);
        }
    }

    fn into_vec(&self) -> Vec<SymbolId> {
        self.map.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>()
    }
}

impl ToString for SymbolMap {
    fn to_string(&self) -> String {
        serde_json::to_string_pretty(&self.map.clone().into_values().collect::<Vec<Symbol>>())
            .unwrap()
    }
}
