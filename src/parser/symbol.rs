///
/// The symbol module is responsible for maintaining a symbol tree
///

static mut VERBOSE: bool = true;

macro_rules! log {
    ($message:expr $(,$arg:expr)*) => {
        unsafe {
            if VERBOSE == true {
                println!($message, $($arg,)*);
            }
        }
    };
}

///
/// SymbolTable is a data structure responsible for managing symbols
/// and pushing and popping scopes, as well as refusing symbols
/// if they overlap in the current scope.
///
#[derive(Clone)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,

    old_table: Option<Box<SymbolTable>>,

    next_offset: u32,

    next_temp: u32,
}

impl SymbolTable {
    /// Returns a new empty symbol table
    pub fn empty() -> SymbolTable {
        SymbolTable {
            symbols: Vec::<Symbol>::new(),
            old_table: None,
            next_offset: 0,
            next_temp: 0,
        }
    }

    /// Consumes self to make it the child of the next scope
    fn child_table(self) -> SymbolTable {
        log!("<YASLC/SymbolTable> Creating child symbol table for table to create new scope.");

        let pointer_old = Box::<SymbolTable>::new(self);

        SymbolTable {
            symbols: Vec::<Symbol>::new(),
            old_table: Some(pointer_old),
            next_offset: 0,
            next_temp: 0,
        }
    }

    /// Adds a symbol given the identifer and type
    pub fn add(&mut self, identifier: String, t: SymbolType) {
        for s in self.symbols.iter() {
            if s.identifier == identifier {
                panic!("<YASLC/SymbolTable> Error: Attempted to insert symbol that already exists in the scope!");
            }
        }

        let o = self.next_offset.clone();
        self.add_symbol(Symbol{
            identifier: identifier,
            symbol_type: t,
            register: 0,
            offset: o,
        });

        self.next_offset += 4;
    }

    /// Adds (binds) a new symbol to the table
    fn add_symbol(&mut self, s: Symbol) {
        self.symbols.insert(0, s);
        log!("<YASLC/SymbolTable> Added new symbol to table, printing...");
        self.print_table();
    }

    /// Get (lookup) a symbol on the table
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

    /// Enters the next table
    pub fn enter(self) -> SymbolTable {
        self.child_table()
    }

    /// Exits the current table, returning the previous
    pub fn exit(self) -> Option<SymbolTable> {
        log!("Table attempting to exit and dereference itself. Printing table.");
        self.print_table();

        match self.old_table {
            Some(b) => Some(*b),
            None => None
        }
    }

    pub fn temp(&mut self) -> Symbol {
        let name = format!("${}", self.next_temp);

        let s = Symbol {
            identifier: name,
            symbol_type: SymbolType::Variable(SymbolValueType::Int),
            offset: self.next_offset,
            register: 0,
        };

        self.next_temp += 1;

        s
    }

    fn print_table(&self) {
        if let Some(ref b) = self.old_table {
            b.print_table();
        }

        print!("Table: [");

        for s in self.symbols.iter() {
            print!("{}", s.identifier);
        }

        println!("]");
    }
}



#[derive(Clone)]
pub struct Symbol {
    identifier: String,
    pub symbol_type: SymbolType,
    offset: u32,
    register: u32,
}

impl Symbol {
    pub fn identifier(&self) -> &String {
        &self.identifier
    }

    pub fn symbol_type(&self) -> &SymbolType {
        &self.symbol_type
    }

    pub fn offset(&self) -> u32 {
        self.offset.clone()
    }

    pub fn register(&self) -> u32 {
        self.register.clone()
    }

    pub fn set_register(&mut self, register: u32) {
        self.register = register;
    }
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
