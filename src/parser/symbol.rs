/// parser/symbol.rs
///
/// The symbol module is responsible for maintaining a symbol tree and making sure
/// it is valid.
///

use std::ops::Index;

/// Set to true if you want the logs of symbol functionality, false otherwise.
static mut VERBOSE: bool = false;

macro_rules! log {
    ($message:expr $(,$arg:expr)*) => {
        unsafe {
            if VERBOSE == true {
                println!($message, $($arg,)*);
            }
        }
    };

    (NNL $message:expr $(,$arg:expr)*) => {
        unsafe {
            if VERBOSE == true {
                print!($message, $($arg,)*);
            }
        }
    }
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

    register: Option<String>,

    register_n: u32,

    next_offset: u32,

    next_temp: u32,

    next_bool_temp: u32,

    next_if_temp: u32,

    next_while_temp: u32,

    proc_stack: Vec<String>,
}

impl SymbolTable {
    /// Returns a new empty symbol table
    pub fn empty() -> SymbolTable {
        SymbolTable {
            symbols: Vec::<Symbol>::new(),
            old_table: None,
            register: None,
            register_n: 0,
            next_offset: 0,
            next_temp: 0,
            next_bool_temp: 0,
            next_if_temp: 0,
            next_while_temp: 0,
            proc_stack: Vec::<String>::new(),
        }
    }

    /// Consumes self to make it the child of the next scope
    fn child_table(self) -> SymbolTable {
        log!("<YASLC/SymbolTable> Creating child symbol table for table to create new scope.");

        let register_n = self.register_n;
        let n_o = self.next_offset;
        let n_t = self.next_temp;
        let n_bt = self.next_bool_temp;
        let n_it = self.next_if_temp;
        let n_wt = self.next_while_temp;
        let ps = self.proc_stack.clone();

        let pointer_old = Box::<SymbolTable>::new(self);

        SymbolTable {
            symbols: Vec::<Symbol>::new(),
            old_table: Some(pointer_old),
            register: None,
            register_n: register_n,
            next_offset: n_o,
            next_temp: n_t,
            next_bool_temp: n_bt,
            next_if_temp: n_it,
            next_while_temp: n_wt,
            proc_stack: ps,
        }
    }

    /// Adds a symbol given the identifer and type
    pub fn add(&mut self, identifier: String, t: SymbolType) {
        for s in self.symbols.iter() {
            if s.identifier == identifier {
                panic!("<YASLC/SymbolTable> Error: Attempted to insert symbol that already exists in the scope!");
            }
        }

        if t == SymbolType::Procedure {
            log!("Found a procedure!");
            self.proc_stack.push(identifier.clone());
        }

        let r = match self.register {
            Some(ref r) => {
                Some(r.clone())
            },
            None => None,
        };

        let o = self.next_offset.clone();

        if t != SymbolType::Procedure {
            self.next_offset += 4;
        }

        self.add_symbol(Symbol{
            identifier: identifier,
            symbol_type: t,
            register: r,
            register_n: 0,
            offset: o,
        });
    }

    /// Adds (binds) a new symbol to the table
    fn add_symbol(&mut self, s: Symbol) {
        self.symbols.insert(0, s);
        log!("<YASLC/SymbolTable> Added new symbol to table, printing...");
        self.log_table();
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

    pub fn enter_proc(self) -> SymbolTable {
        let mut c = self.enter();

        c.register = Some(format!("FP"));
        c.next_offset = 0;
        c.register_n = 0;

        c
    }

    /// Exits the current table, returning the previous
    pub fn exit(self) -> Option<SymbolTable> {
        log!("Table attempting to exit and dereference itself. Printing table.");
        self.log_table();

        let proc_t = self.proc_stack;

        match self.old_table {
            Some(b) => {
                let mut old = *b;
                old.proc_stack = proc_t;
                Some(old)
            },
            None => None
        }
    }

    /// Returns the next temp variable using $(NUMBER) where NUMBER is incremented and
    /// guarenteed to be unique.
    pub fn temp(&mut self, s_type: SymbolType) -> Symbol {
        let name = format!("${}", self.next_temp);

        let s = Symbol {
            identifier: name,
            symbol_type: s_type,
            offset: self.next_offset,
            register_n: 1,
            register: self.register.clone(),
        };

        self.next_temp += 1;
        self.next_offset += 4;

        self.add_symbol(s.clone());

        s
    }

    pub fn up_register(&mut self) {
        let prev = (self.next_offset, self.next_temp);

        self.register_n += 1;
        self.next_offset = 0;
        self.next_temp = 0;
    }

    pub fn bool_temp(&mut self) -> u32 {
        self.next_bool_temp += 1;
        self.next_bool_temp - 1
    }

    pub fn if_temp(&mut self) -> u32 {
        self.next_if_temp += 1;
        self.next_if_temp - 1
    }

    pub fn while_temp(&mut self) -> u32 {
        self.next_while_temp += 1;
        self.next_while_temp - 1
    }

    // pub fn down_register(&mut self) {
    //     if self.register_n <= 0 {
    //         panic!("<YASLC/SymbolTable> Internal error: attempted to move down a register_n when we were already at 0!");
    //     }
    //     self.register_n -= 1;
    //
    //     match self.register_saves.pop() {
    //         Some((offset, temp)) => {
    //             self.next_offset = offset;
    //             self.next_temp = temp;
    //         },
    //         None => panic!("<YASLC/SymbolTable> Tried to move down a register_n but we did not have save data for the previous register_n!"),
    //     }
    // }

    /// Resets the next_offset property.
    pub fn reset_offset(&mut self) {
        self.next_offset = 0;
    }

    pub fn current_proc(&self) -> String {
        if self.proc_stack.len() == 0 {
            return format!("mainblock");
        }

        return self.proc_stack[self.proc_stack.len() - 1].clone();
    }

    /// Prints the table's sub-tables and then itself.
    fn print_table(&self) {
        if let Some(ref b) = self.old_table {
            b.print_table();
        }

        print!("Table: [");

        for s in self.symbols.iter() {
            print!("{}, ", s.identifier);
        }

        println!("]");
    }

    fn log_table(&self) {
        if let Some(ref b) = self.old_table {
            b.log_table();
        }

        log!(NNL "Table: [");

        for s in self.symbols.iter() {
            log!(NNL "{}, ", s.identifier);
        }

        log!("]");
    }
}


/// A single symbol with an identifier, offset on the stack and register_n, as well as a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    /// The identifier for this symbol.
    identifier: String,

    /// The type for this symbol.
    pub symbol_type: SymbolType,

    /// The offest for this symbol.
    offset: u32,

    register: Option<String>,

    /// The register_n for which to offset from for this symbol.
    register_n: u32,
}

impl Symbol {
    pub fn is_temp(&self) -> bool {
        if self.identifier.len() == 0 {
            println!("<YASLC/SymbolTable> Warning, found a symbol with an empty identifier. This is bad.");
            return false;
        }
        self.identifier.index(0..1) == "$"
    }

    pub fn location(&self) -> String {
        let r = match self.register.clone() {
            Some(s) => s,
            None => format!("R{}", self.register_n),
        };
        format!("+{}@{}", self.offset, &r)
    }

    pub fn identifier(&self) -> &String {
        &self.identifier
    }

    pub fn symbol_type(&self) -> &SymbolType {
        &self.symbol_type
    }

    pub fn set_value_type(&mut self, v_type: SymbolValueType) {
        self.symbol_type = match self.symbol_type {
            SymbolType::Variable(_) => SymbolType::Variable(v_type),
            SymbolType::Constant(_) => SymbolType::Constant(v_type),
            _ => panic!("Attempted to set value type for a procedure!"),
        };
    }
}

/// The type of symbol.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SymbolType {
    /// The symbol is a procedure.
    Procedure,

    /// The symbol is a variable.
    Variable(SymbolValueType),

    /// The symbol is a constant.
    Constant(SymbolValueType),
}

/// If the symbol type can have a value, it needs to be typed. SymbolValueType
/// represents different primitive types within YASL.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SymbolValueType {
    Int,
    Bool,
}
