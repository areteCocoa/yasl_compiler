///
/// The symbol module is responsible for maintaining a symbol tree
///

use super::super::lexer::token::{Token};

pub struct SymbolTable {
    symbols: Vec<Symbol>,

    old_table: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    // Returns a new empty symbol table
    pub fn empty() -> SymbolTable {
        SymbolTable {
            symbols: Vec::<Symbol>::new(),
            old_table: None,
        }
    }

    // Adds a symbol given the identifer and type
    fn add(&mut self, identifier: String, t: SymbolType) {
        self.add_symbol(Symbol{
            identifier: identifier,
            symbol_type: t,
        });
    }

    // Adds (binds) a new symbol to the table
    fn add_symbol(&mut self, s: Symbol) {
        self.symbols.insert(0, s);
    }

    // Get (lookup) a symbol on the table
    fn get(&self, name: &str) -> Option<&Symbol> {
        for s in self.symbols.iter() {
            if s.identifier == name {
                return Some(s);
            }
        }

        // If we have a lower table use that
        if let Some(ref b) = self.old_table {
            return (*b).get(name);
        }

        None
    }

    // Enters the next table
    fn enter(self) -> SymbolTable {
        let p = Box::<SymbolTable>::new(self);

        let mut new = SymbolTable::empty();
        new.old_table = Some(p);

        new
    }

    // Exits the current table, returning the previous
    fn exit(self) -> Option<SymbolTable> {
        match self.old_table {
            Some(b) => Some(*b),
            None => None
        }
    }
}

pub struct Symbol {
    identifier: String,
    symbol_type: SymbolType,
}

pub enum SymbolType {
    Procedure,
    Variable,
    Constant
}
