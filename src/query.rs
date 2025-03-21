use oxc_semantic::SymbolId;
use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Query {
    symbol: String, // e.g. call() symbol
    symbol_id: Option<SymbolId>,
    symbol_path: PathBuf, // from ./factory.js file
}

impl Hash for Query {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(symbol_id) = self.symbol_id {
            symbol_id.hash(state);
        } else {
            self.symbol.hash(state);
        }

        self.symbol_path.hash(state);
    }
}

impl Query {
    #[allow(dead_code)]
    pub fn new(symbol_id: SymbolId, symbol_name: String, symbol_path: PathBuf) -> Self {
        Self {
            symbol: symbol_name,
            symbol_id: Some(symbol_id),
            symbol_path,
        }
    }

    pub fn new_with_symbol(symbol: String, symbol_path: PathBuf) -> Self {
        Self {
            symbol,
            symbol_id: None,
            symbol_path,
        }
    }

    pub fn symbol_path(&self) -> &PathBuf {
        &self.symbol_path
    }

    pub fn symbol(&self) -> &String {
        &self.symbol
    }
}
