///
/// The symbol module is responsible for maintaining a symbol tree
///

#[derive(Clone)]
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

    fn child_table(self) -> SymbolTable {
        let pointer_old = Box::<SymbolTable>::new(self);

        SymbolTable {
            symbols: Vec::<Symbol>::new(),
            old_table: Some(pointer_old),
        }
    }

    // Adds a symbol given the identifer and type
    pub fn add(&mut self, identifier: String, t: SymbolType) {
        for s in self.symbols.iter() {
            if s.identifier == identifier {
                panic!("<YASLC/SymbolTable> Error: Attempted to insert symbol that already exists in the scope!");
            }
        }

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
    pub fn get(&self, name: &str) -> Option<&Symbol> {
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
    pub fn enter(self) -> SymbolTable {
        self.child_table()
    }

    // Exits the current table, returning the previous
    pub fn exit(self) -> Option<SymbolTable> {
        println!("Table attempting to exit and dereference itself. Printing table.");
        self.print_table();

        match self.old_table {
            Some(b) => Some(*b),
            None => None
        }
    }

    fn print_table(&self) {
        if let Some(ref b) = self.old_table {
            b.print_table();
        }

        println!("Table:");

        for s in self.symbols.iter() {
            println!("{}", s.identifier);
        }


    }
}



#[derive(Clone)]
pub struct Symbol {
    identifier: String,
    pub symbol_type: SymbolType,
}

#[derive(Clone)]
pub enum SymbolType {
    Procedure,
    Variable(SymbolValueType),
    Constant(SymbolValueType),
}

#[derive(Clone)]
pub enum SymbolValueType {
    Int,
    Bool,
}
