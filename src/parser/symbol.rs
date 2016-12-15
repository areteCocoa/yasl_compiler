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

    // A list of offsets per register
    register_saves: Vec<(u32, u32)>,

    register: u32,

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
            register_saves: Vec::<(u32, u32)>::new(),
            register: 0,
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

        let register = self.register;
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
            register_saves: Vec::<(u32, u32)>::new(),
            register: register,
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
            register: 1,
        };

        self.next_temp += 1;
        self.next_offset += 4;

        self.add_symbol(s.clone());

        s
    }

    pub fn up_register(&mut self) {
        let prev = (self.next_offset, self.next_temp);
        self.register_saves.push(prev);

        self.register += 1;
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
    //     if self.register <= 0 {
    //         panic!("<YASLC/SymbolTable> Internal error: attempted to move down a register when we were already at 0!");
    //     }
    //     self.register -= 1;
    //
    //     match self.register_saves.pop() {
    //         Some((offset, temp)) => {
    //             self.next_offset = offset;
    //             self.next_temp = temp;
    //         },
    //         None => panic!("<YASLC/SymbolTable> Tried to move down a register but we did not have save data for the previous register!"),
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

    pub fn pop_proc(&mut self) {
        self.proc_stack.pop();
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
}


/// A single symbol with an identifier, offset on the stack and register, as well as a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    /// The identifier for this symbol.
    identifier: String,

    /// The type for this symbol.
    pub symbol_type: SymbolType,

    /// The offest for this symbol.
    offset: u32,

    /// The register for which to offset from for this symbol.
    register: u32,
}

impl Symbol {
    pub fn is_temp(&self) -> bool {
        if self.identifier.len() == 0 {
            panic!("<YASLC/SymbolTable> Warning, found a symbol with an empty identifier. This is bad.");
            return false;
        }
        self.identifier.index(0..1) == "$"
    }

    pub fn location(&self) -> String {
        format!("+{}@R{}", self.offset, self.register)
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
