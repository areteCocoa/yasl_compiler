/// parser/mod.rs
///
/// The parser module is responsible for syntax parsing of a set of compiler tokens
/// using an LL(1) parser, as well as using the expression submodule to parse expressions.
///
/// The parser generates code for the final output file.

pub mod symbol;
mod expression;
mod file_generator;

pub use super::lexer::{Token, TokenType, KeywordType};

pub use self::symbol::{Symbol, SymbolTable, SymbolType, SymbolValueType};
use self::file_generator::file_from;
use self::expression::ExpressionParser;

/// Set true if you want the parser to log all its progress, false otherwise.
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

/// A macro to check the token and invoke closures based on the success or error
/// of the token check.
macro_rules! c_token {
    // Checks and returns unexpected if it was unexpected
    ($_self:expr, $t:expr) => {
        match $_self.check($t) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        }
    };

    // Checks and calls fail if it is unexpected
    ($_self:expr, $t:expr, $fail:expr) => {
        match $_self.check($t) {
            ParserState::Continue => {},
            _ => $fail,
        }
    };

    // Checks and calls success if a success and fail if it fails
    ($_self:expr, $t:expr, $fail:expr, $success:expr) => {
        match $_self.check($t) {
            ParserState::Continue  => $success,
            _ => $fail,
        }
    };
}

/// A macro to check an expression and call optional closures for fail and success cases.
macro_rules! c_exp {
    // Checks and returns unexpected if it was unexpected
    ($e:expr) => {
        match $e {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        }
    };

    // Checks and calls fail if it is unexpected
    ($e:expr, $fail:expr) => {
        match $e {
            ParserState::Continue => {},
            _ => $fail,
        }
    };

    // Checks and calls success if a success and fail if it fails
    ($e:expr, $fail:expr, $success:expr) => {
        match $e {
            ParserState::Continue  => $success,
            _ => $fail,
        }
    };
}

/// The Parser struct can check syntax for a set of tokens for validity as well as generate
/// the final code for them.
pub struct Parser {
    /// The set of tokens for this Parser.
    tokens: Vec<Token>,

    /// The last expression symbol from the evaluated expression
    last_expression: Option<Symbol>,

    /// The last popped token.
    last_token: Option<Token>,

    /// The symbol table associated with this parser.
    symbol_table: SymbolTable,

    /// The stack of tokens used with the expression parser.
    //stack: Vec<Token>,

    /// The vector of strings for output to the file.
    commands: CommandBuilder,

    /// A vector of declarations for output to the file.
    declarations: Vec<String>,
}

/// The parser is implemented with some convenience functions for many rules. However,
/// some rules still have to checked "manually." For any rule that can be accessed from a
/// rule that can go to empty, you must check the first token to make sure you're in the
/// correct rule.
impl Parser {

    /// Returns a new parser given the input tokens.
    pub fn new_with_tokens(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,

            last_token: None,

            last_expression: None,

            symbol_table: SymbolTable::empty(),

            commands: CommandBuilder::new(),

            declarations: Vec::<String>::new(),
        }
    }

    /// Starts to parse on the set of input tokens.
    pub fn parse(&mut self) -> ParserResult {
        match self.program() {
            ParserState::Done(r) => {
                match r {
                    ParserResult::Success => {
                        log!("<YASLC/Parser> Correctly parsed YASL program file.");

                        // Get the number of declarations
                        let n_decl = self.declarations.len();

                        // Move the SP based on the number of declarations
                        self.declarations.push(format!("addw #{}, SP", n_decl * 4));
                        self.declarations.push(format!(""));

                        // Create one list of commands
                        self.declarations.append(&mut self.commands.commands);

                        // "Fix" commands with prepends and appends
                        self.declarations.insert(0, format!("$main movw SP R0"));
                        self.declarations.insert(0, format!("$junk #1"));
                        self.declarations.insert(0, format!(": Initialize junk variable and setup the stack"));

                        match file_from(self.declarations.clone()) {
                            Ok(f) => {
                                log!("<YASLC/Parser> Successfully wrote file {:?}!", f);
                            },
                            Err(e) => {
                                log!("<YASLC/Parser> Error writing file: {:?}", e);
                            },
                        };

                        return ParserResult::Success;
                    },
                    // It was not a success, figure out what went wrong.
                    _ => {
                        // Get the error token
                        if let Some(t) = self.last_token() {
                            println!("<YASLC/Parser> Error: Unexpected token at ({}, {}) of type: {}", t.line(), t.column(), t.token_type());
                        } else {
                            println!("<YASC/Parser> Internal error: Could not find the error token, we don't know what went wrong.");
                        }
                        return ParserResult::Unexpected;
                    }
                }
            }

            ParserState::Continue => {
                if let Some(t) = self.last_token() {
                    println!("<YASLC/Parser> Unexpected end of file at ({}, {}): {}", t.line(), t.column(), t.token_type());
                } else {
                    println!("<YASC/Parser> Unexpected end of file. No token found, we don't know what went wrong.");
                }
                return ParserResult::Unexpected;
            }
        }
    }

    /// Pops the front token off the stack of tokens and returns it.
    fn next_token(&mut self) -> Token {
        if self.tokens.len() == 0 {
            panic!("Unexpected end of file!");
        }
        let t = self.tokens.remove(0);

        self.last_token = Some(t.clone());

        t
    }

    /// Returns a copy of the last token popped.
    fn last_token(&mut self) -> Option<Token> {
        self.last_token.clone()
    }

    /// Inserts the last token popped into the token set.
    fn insert_last_token(&mut self) {
        if let Some(a) = self.last_token() {
            self.tokens.insert(0, a);
            self.last_token = None;
        } else {
            log!("<YASLC/Parser> Internal warning: Attempted to insert the last token into the parser but there is no last token!");
        }
    }

    /// Checks the next token for the token type t and returns the parser state (continue or done)
    /// based on the input.
    fn check(&mut self, t: TokenType) -> ParserState {
        let token = self.next_token();

        log!("<YASLC/Parser> Checking if token {} is of type {}.", token, t);
        log!("\t\t\t {} tokens left in vector.", self.tokens.len());

        self.check_token(t, token)
    }

    /// Checks if the token is the correct type and returns Continue if it is, Unexpected token
    /// otherwise.
    fn check_token(&mut self, t: TokenType, token: Token) -> ParserState {
        match token.is_type(t) {
            true => ParserState::Continue,
            false => ParserState::Done(ParserResult::Unexpected),
        }
    }

    //. Checks the token for the first token type t1. If it fails it checks the token for type t2.
    //. Returns success if either is the type of token.
    fn check_and_then_check(&mut self, t1: TokenType, t2: TokenType)
        -> (ParserState, Option<TokenType>) {
        match self.check(t1.clone()) {
            ParserState::Continue => (ParserState::Continue, Some(t1)),
            ParserState::Done(_) => {
                self.insert_last_token();
                (self.check(t2.clone()), Some(t2))
            },
        }
    }

    /// Adds the string command to the list of commands.
    fn push_command(&mut self, command: String) {
        log!("<YASLC/Parser> Adding command to list of output: \'{}\'", command);
        self.commands.push_command(command);
    }

    fn push_prefix(&mut self, prefix: String) -> String {
        self.commands.set_prefix(prefix)
    }

    /// Adds the print command, which is a series of single character outputs.
    fn add_print_command(&mut self, print_message: &str) {
        let mut i = 0;
        for c in print_message.chars() {
            if i != 0 && i != print_message.len()-1 {
                self.push_command(format!("outb #{}", c as u8));
            }
            i += 1;
        }
        self.push_command(format!("outb #10"));
    }

    /**
     * YASL Context free grammar rules
     */

    /*
     *  PROGRAM rule
     */
    fn program(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PROGRAM rule.");

        c_token!(self, TokenType::Keyword(KeywordType::Program));
        c_token!(self, TokenType::Identifier);
        c_token!(self, TokenType::Semicolon);

        c_exp!(self.block());

        c_token!(self, TokenType::Period, ParserState::Continue, {
            log!("<YASLC/Parser> Exiting Parser because we found the final period.");
            self.push_command(format!("inb $junk"));
            self.push_command(format!("end"));
            ParserState::Done(ParserResult::Success)
        })
    }

    // BLOCK rule
    fn block (&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting BLOCK rule.");

        let proc_t = self.symbol_table.current_proc();

        if self.commands.prefix.is_some() {
            self.commands.push_useless();
        }

        if proc_t != "mainblock" {
            self.push_command(format!(": Block {}", proc_t));
            self.commands.set_prefix(format!("${}", proc_t));
        }

        c_exp!(self.consts());
        c_exp!(self.vars());

        if proc_t == "mainblock" {
            self.push_command(format!(": Jump to block {} of execution", proc_t));
            self.push_command(format!("jmp ${}", proc_t));
            self.push_command(format!(""));
        }

        c_exp!(self.procs());

        c_token!(self, TokenType::Keyword(KeywordType::Begin));
        if proc_t == "mainblock" {
            self.push_command(format!(": Block {}", proc_t));
            self.commands.set_prefix(format!("${}", proc_t));
        }

        c_exp!(self.statements());

        match self.check(TokenType::Keyword(KeywordType::End)) {
            ParserState::Continue => {
                if proc_t != "mainblock" {
                    self.push_command(format!("ret\n: end {}\n", proc_t));
                }

                ParserState::Continue
            },
            x => x,
        }
    }

    /*
     *  CONSTS rule
     */
    fn consts(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting CONSTS rule.");

        match self.token_const() {
            ParserState::Continue => self.consts(),
            ParserState::Done(ParserResult::Incorrect) => {
                self.insert_last_token();
                ParserState::Continue
            },
            _ => ParserState::Done(ParserResult::Unexpected),
        }
    }

    // CONST rule
    fn token_const(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting CONST rule.");

        c_token!(self, TokenType::Keyword(KeywordType::Const),
            return ParserState::Done(ParserResult::Incorrect));

        let id = match self.check(TokenType::Identifier) {
            ParserState::Continue => {
                self.last_token().unwrap().lexeme()
            },
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        c_token!(self, TokenType::Assign);

        let (t, v) = match self.check(TokenType::Number) {
            ParserState::Continue => {
                let l = self.last_token().unwrap();

                // If the lexeme is numeric it's a number, otherwise if its "true"/"false its a boolean"
                // if its neither then crash
                match l.lexeme().parse::<i32>() {
                    Ok(n) => {
                        // Its a number
                        (SymbolValueType::Int, n)
                    },
                    Err(_) => {
                        // It is not a number, check if it is a boolean
                        if l.lexeme() == "true" {
                            (SymbolValueType::Bool, 1)
                        } else if l.lexeme() == "false" {
                            (SymbolValueType::Bool, 0)
                        } else {
                            // We don't know what it is, crash.
                            panic!("<YASLC/Parser> Invalid constant value: {}", l.lexeme());
                        }
                    }
                }
            },

            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        self.symbol_table.add(id.clone(), SymbolType::Constant(t));
        match self.symbol_table.get(&*id) {
            Some(s) => {
                // If it is a constant then set the value
                let c = format!("movw #{} {}", v, s.location());
                if self.symbol_table.current_proc() == "mainblock" {
                    self.declarations.push(c);
                } else {
                    self.commands.push_command(format!("movw #{} {}", v, s.location()));
                }
            },
            None => {
                panic!("Internal error with the symbol table.");
            }
        }


        c_token!(self, TokenType::Semicolon,
            ParserState::Done(ParserResult::Unexpected),
            ParserState::Continue)
    }

    // VARS rule
    fn vars(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting VARS rule.");

        match self.var() {
            ParserState::Continue => self.vars(),
            ParserState::Done(ParserResult::Incorrect) => {
                self.insert_last_token();
                ParserState::Continue
            },
            _ => {
                ParserState::Done(ParserResult::Unexpected)
            },
        }
    }

    // VAR rule
    fn var(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting VAR rule.");

        c_token!(self, TokenType::Keyword(KeywordType::Var), return ParserState::Done(ParserResult::Incorrect));

        let id = match self.check(TokenType::Identifier) {
            ParserState::Continue => {self.last_token().unwrap().lexeme()},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        c_token!(self, TokenType::Colon);

        let t = match self.token_type() {
            ParserState::Continue => {
                match self.last_token().unwrap().token_type() {
                    TokenType::Keyword(KeywordType::Bool) => {
                        SymbolValueType::Bool
                    },
                    TokenType::Keyword(KeywordType::Int) => {
                        SymbolValueType::Int
                    },
                    _ => {
                        println!("<YASLC/Parser> Error: Unrecognized type for var found {}.", self.last_token().unwrap());
                        return ParserState::Done(ParserResult::Unexpected);
                    }
                }

            },
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        self.symbol_table.add(id.clone(), SymbolType::Variable(t));
        match self.symbol_table.get(&*id) {
            Some(s) => {
                // Initialize the value as 0
                self.declarations.push(format!("movw #0 {}", s.location()));
            },
            None => {
                panic!("Internal error with the symbol table.");
            }
        }

        self.check(TokenType::Semicolon)
    }

    // TYPE rule
    fn token_type(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting TYPE rule.");

        self.check_and_then_check(TokenType::Keyword(KeywordType::Int),
            TokenType::Keyword(KeywordType::Bool)).0
    }

    // PROCS rule
    fn procs(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PROCS rule.");

        match self.token_proc() {
            ParserState::Continue => self.procs(),
            ParserState::Done(ParserResult::Incorrect) => {
                self.insert_last_token();
                ParserState::Continue
            },
            _ => {
                ParserState::Done(ParserResult::Unexpected)
            },
        }
    }

    // PROC rule
    fn token_proc(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PROC rule.");

        //let t = self.symbol_table.temp(SymbolType::Variable(SymbolValueType::Int));

        match self.check(TokenType::Keyword(KeywordType::Proc)) {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Incorrect),
        };

        self.symbol_table = self.symbol_table.clone().enter_proc();

        let id = match self.check(TokenType::Identifier) {
            ParserState::Continue => {
                self.last_token().unwrap().lexeme()
            },
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        self.symbol_table.add(id, SymbolType::Procedure);

        c_exp!(self.param_list());

        c_token!(self, TokenType::Semicolon);

        c_exp!(self.block());

        let r = match self.check(TokenType::Semicolon) {
            ParserState::Continue => ParserState::Continue,
            _ => ParserState::Done(ParserResult::Unexpected),
        };

        self.symbol_table = match self.symbol_table.clone().exit(){
            Some(s) => {
                s
            }
            None => {
                panic!("<YASLC/Parser> A symbol table has been popped where it shouldn't have been and we're in big trouble.");
            }
        };

        r
    }

    // PARAM-LIST rule
    fn param_list(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PARAM-LIST rule.");

        match self.check(TokenType::LeftParen) {
            ParserState::Continue => ParserState::Continue,
            _ => {
                self.insert_last_token();
                return ParserState::Continue;
            },
        };

        c_exp!(self.params());

        c_token!(self, TokenType::RightParen,
            ParserState::Done(ParserResult::Unexpected),
            ParserState::Continue)
    }

    // PARAMS rule
    fn params(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PARAMS rule.");

        c_exp!(self.param());

        match self.follow_param() {
            ParserState::Continue => ParserState::Continue,
            ParserState::Done(ParserResult::Incorrect) => {
                self.insert_last_token();
                ParserState::Continue
            }
            _ => ParserState::Done(ParserResult::Unexpected)
        }
    }

    // FOLLOW_PARAM rule
    fn follow_param(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-PARAM rule.");

        match self.check(TokenType::Comma) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Incorrect),
        };

        match self.params() {
            ParserState::Continue => ParserState::Continue,
            ParserState::Done(a) => ParserState::Done(a),
        }
    }

    // PARAM rule
    fn param(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PARAM rule.");

        c_token!(self, TokenType::Identifier);

        c_token!(self, TokenType::Colon);

        c_exp!(self.token_type(), ParserState::Done(ParserResult::Unexpected),
            ParserState::Continue)
    }

    // STATEMENTS rule
    fn statements(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting STATEMENTS rule.");

        match self.statement() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.statement_tail() {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Unexpected),
        }
    }

    // STATEMENT-TAIL rule
    fn statement_tail(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting STATEMENT-TAIL rule.");

        match self.check(TokenType::Semicolon) {
            ParserState::Continue => {},
            _ => {
                self.insert_last_token();
                return ParserState::Continue;
            },
        };

        c_exp!(self.statement());

        self.statement_tail()
    }

    // STATEMENT rule
    // Statement rule is special because there are so many types of statements that we must
    // be more explicit with definitions.
    fn statement(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting STATEMENT rule.");

        let token = self.next_token();

        match self.check_token(TokenType::Keyword(KeywordType::If), token.clone()) {
            ParserState::Continue => {
                let i_temp = self.symbol_table.if_temp();
                match self.expression() {
                    ParserState::Continue => {
                        // Get the value of the boolean expression and compare it to 0. If it is
                        // eq to 0 then go to else
                        let s = match self.last_expression {
                            Some(ref s) => s.clone(),
                            None => {
                                panic!("Attempted to ge the last expression for an if statement but it isn't there!");
                            }
                        };

                        self.commands.push_command(format!("cmpw #0 {}", s.location()));
                        self.commands.push_command(format!("beq $if_else{}", i_temp));
                    },
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                match self.check(TokenType::Keyword(KeywordType::Then)) {
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                match self.statement() {
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                // Statements have ended, jump to end,
                // and prepend next command with $if_else{}
                self.commands.push_command(format!("jmp $end_if{}", i_temp));
                self.commands.set_prefix(format!("$if_else{}", i_temp));

                match self.follow_if() {
                    ParserState::Continue => {
                        self.commands.set_prefix(format!("$end_if{}", i_temp));
                        return ParserState::Continue;
                    },
                    x => return x,
                };
            },
            _ => {},
        };

        match self.check_token(TokenType::Keyword(KeywordType::While), token.clone()) {
            ParserState::Continue => {
                // Setup the starting marker
                let w_temp = self.symbol_table.while_temp();
                self.push_command(format!("\n: while loop {}", w_temp));
                self.push_prefix(format!("$b_while{}", w_temp));

                // Evaluate the expression
                match self.expression() {
                    ParserState::Continue => {
                        // Get the value of the boolean expression and compare it to 0, leave the
                        // loop. Continue otherwise.
                        let s = match self.last_expression {
                            Some(ref s) => s.clone(),
                            None => {
                                panic!("Attempted to ge the last expression for a while statement but it isn't there!");
                            }
                        };

                        self.commands.push_command(format!("cmpw #0 {}", s.location()));
                        self.commands.push_command(format!("beq $e_while{}", w_temp));
                    },
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                match self.check(TokenType::Keyword(KeywordType::Do)) {
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                // Code for the statement generates by itself
                match self.statement() {
                    ParserState::Continue => {
                        self.commands.push_command(format!("jmp $b_while{}", w_temp));
                        self.commands.set_prefix(format!("$e_while{}", w_temp));
                        return ParserState::Continue;
                    },
                    x => return x,
                };
            },
            _ => {},
        };

        match self.check_token(TokenType::Keyword(KeywordType::Begin), token.clone()) {
            ParserState::Continue => {
                return self.follow_begin();
            },
            _ => {},
        };

        match self.check_token(TokenType::Identifier, token.clone()) {
            ParserState::Continue => {
                return self.follow_id();
            },
            _ => {},
        };

        match self.check_token(TokenType::Keyword(KeywordType::Prompt), token.clone()) {
            ParserState::Continue => {
                match self.check(TokenType::String) {
                    ParserState::Continue => {
                        // Output the string
                        let l = self.last_token().unwrap().lexeme();
                        self.add_print_command(&*l)
                    },
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                match self.follow_prompt() {
                    ParserState::Continue => {
                        // Check if we're taking input or not

                        // Get the variable, whether it's junk or an actual varible
                        let (c, v) = match self.last_token() {
                            Some(t) => {
                                // If there's a value then we successfully parsed the Identifier
                                log!("<YASLC/Parser> Parsed PROMPT with identifier, adding to compiled file.");
                                match self.symbol_table.get(&*t.lexeme()) {
                                    Some(s) => {
                                        ("inw", s.location())
                                    },
                                    None => {
                                        ("inb", format!("$junk"))
                                    }
                                }
                            },
                            None => {
                                // If there's no value, we have no identifier
                                log!("<YASLC/Parser> Parsed PROMPT without identifier, using $junk and adding to compiled file.");
                                ("inb", format!("$junk"))
                            }
                        };

                        // Prompt for the variable
                        log!("<YASLC/Parser> Adding prompt command for variable {}", v);

                        self.push_command(format!("{} {}", c, v));

                        return ParserState::Continue;
                    },
                    a => return a,
                }
            },
            _ => {},
        };

        match self.check_token(TokenType::Keyword(KeywordType::Print), token.clone()) {
            ParserState::Continue => {
                return self.follow_print();
            },
            _ => {},
        };

        ParserState::Done(ParserResult::Unexpected)
    }

    // FOLLOW-IF rule
    fn follow_if(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-IF rule.");

        match self.check(TokenType::Keyword(KeywordType::Else)) {
            ParserState::Continue => self.statement(),
            _ => {
                self.insert_last_token();
                ParserState::Continue
            }
        }
    }

    // FOLLOW-BEGIN rule
    fn follow_begin(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-BEGIN rule.");

        match self.statement() {
            ParserState::Continue => {},
            _ => {
                self.insert_last_token();
                return self.check(TokenType::Keyword(KeywordType::End));
            }
        };

        c_exp!(self.statement_tail());

        self.check(TokenType::Keyword(KeywordType::End))
    }

    // FOLLOW-ID rule
    fn follow_id(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-ID rule.");

        // Get the identifier
        let id = self.last_token().unwrap().lexeme();

        // Are we assigning?
        match self.check(TokenType::Assign) {
            ParserState::Continue => {
                match self.expression() {
                    ParserState::Continue => {
                        let f = match self.last_expression {
                            Some(ref e) => {
                                e.clone()
                            },
                            None => {
                                panic!("<YASLC/Parser> Warning: attempted to use expression to set variable but the expression parser is missing!");
                            }
                        };

                        self.last_expression = None;

                        // Move the value of the expression to the identifier
                        let id_symbol = self.symbol_table.get(&*id).unwrap().clone();

                        // Check that we're assigning to a variable
                        match id_symbol.symbol_type {
                            SymbolType::Variable(_) => {},
                            SymbolType::Constant(_) => {
                                println!("<YASLC/Parser> Attempted to assign a value to a constant!");
                                return ParserState::Done(ParserResult::Unexpected);
                            },
                            SymbolType::Procedure => {
                                println!("<YASLC/Parser> Attempted to assign a value to a procedure!");
                                return ParserState::Done(ParserResult::Unexpected);
                            },
                        }

                        // Check that we're assigning to the same type
                        match &id_symbol.symbol_type {
                            &SymbolType::Variable(ref v1) | &SymbolType::Constant(ref v1) => {
                                match &f.symbol_type {
                                    &SymbolType::Variable(ref v2) | &SymbolType::Constant(ref v2) => {
                                        if v1 != v2 {
                                            println!("<YASLC/Parser> Attempted to assign a value to a variable who's type is not the same!");
                                            println!("<YASLC/Parser> Variable is type {:?} and value is type {:?}.", id_symbol.symbol_type, f.symbol_type);
                                            return ParserState::Done(ParserResult::Unexpected)
                                        }
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        };

                        // Add the command
                        // TODO: If you wanted to use more registers, this would need to be overriden to use f.register
                        self.push_command(format!("movw +0@R1 {}", id_symbol.location()));

                        return ParserState::Continue;
                    },
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };
            },
            _ => {},
        };

        // All execution around assigning does not reach this point.
        self.insert_last_token();

        // We're dealing with a proc that may have arguments
        match self.check(TokenType::LeftParen) {
            ParserState::Continue => {
                match self.expression() {
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                match self.follow_expression() {
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                return self.check(TokenType::RightParen);
            }

            _ => {
                // It does not, but it should have a semi
                self.insert_last_token();
                match self.check(TokenType::Semicolon) {
                    ParserState::Continue => {
                        // Call the procedure
                        self.push_command(format!("call #{} ${}", 0, id));
                        // TODO: Move the SP, push arguments, etc
                    },
                    _ => {
                        // Check if it is an end token
                        self.insert_last_token();
                        match self.check(TokenType::Keyword(KeywordType::End)) {
                            ParserState::Continue => {
                                self.insert_last_token();

                                // Call the proc
                                self.push_command(format!("call #{} ${}", 0, id));
                                // TODO: Move the SP, push arguments, etc
                            },
                            x => return x,
                        };
                    }
                };
            },
        };

        self.insert_last_token();

        ParserState::Continue
    }

    // FOLLOW-EXPRESSION rule
    fn follow_expression(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-EXPRESSION rule.");

        match self.check(TokenType::Comma) {
            ParserState::Continue => {},
            _ => {
                self.insert_last_token();
                return ParserState::Continue;
            },
        };

        c_exp!(self.expression());

        self.follow_expression()
    }

    // FOLLOW-PROMPT rule
    fn follow_prompt(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-PROMPT rule.");

        match self.check(TokenType::Comma) {
            ParserState::Continue => {},
            _ => {
                // It was not, we ignore the input
                self.insert_last_token();
                return ParserState::Continue;
            },
        };

        self.check(TokenType::Identifier)
    }

    // FOLLOW-PRINT
    fn follow_print(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-PRINT rule.");

        match self.check(TokenType::String) {
            ParserState::Continue => {
                // It is a String

                let m = self.last_token().unwrap().lexeme();
                self.add_print_command(&*m);

                log!("<YASLC/Parser> Successfully parsed print statement, compiling to file.");

                return ParserState::Continue
            },
            _ => self.insert_last_token(),
        }

        log!("<YASLC/Parser> Adding print statement waiting for expression.");
        match self.expression() {
            ParserState::Continue => {
                let f = if let Some(ref e) = self.last_expression {
                    e.clone()
                } else {
                    println!("<YASLc/Parser> Expected to find an expression parser but it went missing!");
                    return ParserState::Done(ParserResult::Unexpected);
                };

                self.push_command(format!("outw {}", f.location()));
                self.push_command(format!("outb #10"));

                self.last_expression = None;

                return ParserState::Continue;
            },
            _ => return ParserState::Done(ParserResult::Unexpected),
        };
    }

    fn expression(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting EXPRESSION rule.");

        if self.commands.prefix.is_none() {
            self.push_command(format!(""));
        }
        self.push_command(format!("movw SP R1"));

        let mut stack = Vec::<Token>::new();

        while self.tokens.is_empty() == false {
            let t = self.tokens.remove(0);
            match t.token_type() {
                TokenType::Semicolon | TokenType::Keyword(KeywordType::Do)
                | TokenType::Keyword(KeywordType::Then) | TokenType::Keyword(KeywordType::End)
                | TokenType::RightParen | TokenType::Keyword(KeywordType::Else) => {
                    // We can exit because it is the end of the expression
                    log!("<YASLC/Parser> Exiting EXPRESSION rule because we found a {} token.", t);

                    self.tokens.insert(0, t);
                    return self.parse_expression_tokens(stack);
                }
                _ => {
                    stack.push(t);
                }
            };
        }

        log!("<YASLC/Parser> Exiting EXPRESSION rule because unexpectedly we ran out of tokens.");

        ParserState::Done(ParserResult::Unexpected)
    }

    fn parse_expression_tokens(&mut self, tokens: Vec<Token>) -> ParserState {
        let mut comment = String::new();
        comment.push_str(&*"expression: ");
        for t in tokens.iter() {
            comment.push_str(&*format!("{} ", t.lexeme()));
        }

        match ExpressionParser::new(self.symbol_table.clone(), tokens) {
            Some(e) => {
                log!("<YASLC/Parser> Expression parser successfully exited!");

                // Parse through the tokens
                match e.parse() {
                    Ok((f_symbol, commands)) => {
                        let _ = self.symbol_table.bool_temp();

                        self.commands.push_command(format!(": {}", comment));

                        // Add the commands to this list of commands
                        self.commands.push_builder(commands);

                        // Reset the symbol table
                        self.symbol_table.reset_offset();

                        // Set the expression parser to our field and continue
                        self.last_expression = Some(f_symbol);
                        ParserState::Continue
                    },
                    Err(e) => {
                        log!("<YASLC/Parser> Expression parser was not successful: {}", e);
                        ParserState::Done(ParserResult::Unexpected)
                    }
                }
            },
            None => {
                log!("<YASLC/Parser> Expression parser was not in initialization!");
                ParserState::Done(ParserResult::Unexpected)
            }
        }
    }
}

/// The state of the parser, whether it should continue or if it is done and has a result.
enum ParserState {
    /// The parser should continue and is expecting more tokens.
    Continue,

    /// The parser has finished and is returning the result.
    Done(ParserResult)
}

/// The result of a finished parser.
pub enum ParserResult {
    /// The parser should continue parsing starting with the next token.
    Success,

    /// The parser found the incorrect token but it is not unexpected and should be pushed
    /// back onto the stack and try with a different rule.
    Incorrect,

    // The parser reached an unexpected token, should return an error and stop.
    Unexpected,
}

pub struct CommandBuilder {
    commands: Vec<String>,

    prefix: Option<String>,
}

impl CommandBuilder {
    fn new() -> CommandBuilder {
        CommandBuilder {
            commands: Vec::<String>::new(),
            prefix: None,
        }
    }

    fn push_command(&mut self, command: String) {
        match self.prefix {
            Some(ref s) => {
                log!("Pushing prefix with command: {} {}", s, command);
                self.commands.push(format!("{} {}", s, command));
            },
            None => {
                log!("Pushing command: {}", command);
                self.commands.push(command);
            }
        };
        self.prefix = None;
    }

    fn set_prefix(&mut self, prefix: String) -> String {
        if self.prefix.is_some() {
            self.push_command(format!("movw R0 R0"));
        }
        self.prefix = Some(prefix.clone());
        prefix
    }

    fn push_useless(&mut self) {
        self.push_command(format!("movw R0 R0"));
    }

    // fn prepend_last(&mut self, prefix: String) {
    //     let old = match self.commands.pop() {
    //         Some(s) => {
    //             s
    //         },
    //         None => {
    //             log!("Warning: Command builder tried to prepend the last command but there was none! Setting prefix...");
    //             self.set_prefix(prefix);
    //             return;
    //         }
    //     };
    //
    //     let new = format!("{}{}", prefix, old);
    //     self.commands.push(new);
    // }

    fn push_builder(&mut self, builder: CommandBuilder) {
        for c in builder.commands {
            self.push_command(c);
        }

        match builder.prefix {
            Some(s) => {self.set_prefix(s);},
            None => {}
        };
    }
}
