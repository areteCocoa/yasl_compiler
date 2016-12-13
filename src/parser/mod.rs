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

#[allow(unused_imports)]
pub use self::symbol::{Symbol, SymbolTable, SymbolType, SymbolValueType};
use self::file_generator::file_from;
use self::expression::ExpressionParser;

/// Set true if you want the parser to log all its progress, false otherwise.
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

#[allow(dead_code)]

/// The Parser struct can check syntax for a set of tokens for validity as well as generate
/// the final code for them.
pub struct Parser {
    /// The set of tokens for this Parser.
    tokens: Vec<Token>,

    /// The last expression stack from the evaluated expression.
    e_parser: Option<ExpressionParser>,

    /// The last popped token.
    last_token: Option<Token>,

    /// The symbol table associated with this parser.
    symbol_table: SymbolTable,

    /// The stack of tokens used with the expression parser.
    stack: Vec<Token>,

    /// The vector of strings for output to the file.
    commands: Vec<String>,

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

            e_parser: None,

            symbol_table: SymbolTable::empty(),

            stack: Vec::<Token>::new(),

            commands: Vec::<String>::new(),

            declarations: Vec::<String>::new(),
        }
    }

    /// Starts to parse on the set of input tokens.
    pub fn parse(&mut self) {
        match self.program() {
            ParserState::Done(r) => {
                match r {
                    ParserResult::Success => {
                        println!("<YASLC/Parser> Correctly parsed YASL program file.");

                        self.declarations.append(&mut self.commands);
                        // Fix the first command to start with $main
                        let first = match self.declarations.get(0) {
                            Some(s) => s.clone(),
                            None => panic!("Attempted to prepend the first command but there is no first command!"),
                        };
                        let new_first = format!("$main {}", first);

                        self.declarations[0] = new_first;

                        match file_from(self.declarations.clone()) {
                            Ok(f) => {
                                log!("<YASLC/Parser> Successfully wrote file {:?}!", f);
                            },
                            Err(e) => {
                                log!("<YASLC/Parser> Error writing file: {:?}", e);
                            },
                        };
                    },
                    // It was not a success, figure out what went wrong.
                    _ => {
                        // Get the error token
                        if let Some(t) = self.last_token() {
                            println!("<YASLC/Parser> Error: Unexpected token at ({}, {}) of type: {}", t.line(), t.column(), t.token_type());
                        } else {
                            println!("<YASC/Parser> Internal error: Could not find the error token, we don't know what went wrong.");
                        }
                    }
                }
            }

            ParserState::Continue => {
                if let Some(t) = self.last_token() {
                    println!("<YASLC/Parser> Unexpected end of file at ({}, {}): {}", t.line(), t.column(), t.token_type());
                } else {
                    println!("<YASC/Parser> Unexpected end of file. No token found, we don't know what went wrong.");
                }
            }
        }
    }

    /// Pops the front token off the stack of tokens and returns it.
    fn next_token(&mut self) -> Token {
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
    fn add_command(&mut self, command: &str) {
        let s = self.commands.len();
        log!("<YASLC/Parser> Adding command to list of output: \'{}\'", command);
        self.commands.push(command.to_string());
    }

    /// Adds the print command, which is a series of single character outputs.
    fn add_print_command(&mut self, print_message: &str) {
        let mut i = 0;
        for c in print_message.chars() {
            if i != 0 && i != print_message.len()-1 {
                // TODO: Treat non-alphabet characters special
                self.add_command(&*format!("outb ^{}", c));
            }
            i += 1;
        }
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
            self.add_command("end");
            ParserState::Done(ParserResult::Success)
        })
    }

    // BLOCK rule
    fn block (&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting BLOCK rule.");

        c_exp!(self.consts());
        c_exp!(self.vars());
        c_exp!(self.procs());

        c_token!(self, TokenType::Keyword(KeywordType::Begin));

        c_exp!(self.statements());
        c_token!(self, TokenType::Keyword(KeywordType::End),
            ParserState::Done(ParserResult::Unexpected),
            ParserState::Continue)
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

        // TODO: Implement detecting value type based on lexeme
        // let (t, _) = match self.check(TokenType::Number) {
        //     ParserState::Continue => {
        //         let l = self.last_token().unwrap();
        //
        //         match l.token_type() {
        //             TokenType::Keyword(KeywordType::Int) => {
        //                 let n = match l.lexeme().parse::<i32>() {
        //                     Ok(x) => x,
        //                     Err(_) => {
        //                         println!("<YASLC/Parser> Token had type int but could not convert to digit!");
        //                         return ParserState::Done(ParserResult::Unexpected);
        //                     }
        //                 };
        //                 (SymbolValueType::Int, n)
        //             },
        //             TokenType::Keyword(KeywordType::Bool) => {
        //                 match &*l.lexeme() {
        //                     "true" => (SymbolValueType::Bool, 1),
        //                     "false" => (SymbolValueType::Bool, 0),
        //                     _ => {
        //                         println!("<YASLC/Parser> Token had type int but could not convert to digit!");
        //                         return ParserState::Done(ParserResult::Unexpected);
        //                     }
        //                 }
        //             },
        //             _ => {
        //                 println!("Something bad happened.");
        //                 return ParserState::Done(ParserResult::Unexpected);
        //             }
        //         }
        //     },
        //
        // };

        c_token!(self, TokenType::Number);

        self.symbol_table.add(id.clone(), SymbolType::Constant(SymbolValueType::Int));
        let value = self.last_token().unwrap().lexeme();
        match self.symbol_table.get(&*id) {
            Some(s) => {
                // If it is a constant then set the value
                self.declarations.push(format!("movw ^{} +{}@R{}", value, s.offset(), s.register()));
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
                    TokenType::Keyword(KeywordType::Bool) => SymbolValueType::Bool,
                    TokenType::Keyword(KeywordType::Int) => SymbolValueType::Int,
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
                self.declarations.push(format!("movw ^0 +{}@R{}", s.offset(), s.register()));
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

        self.symbol_table = self.symbol_table.clone().enter();

        match self.check(TokenType::Keyword(KeywordType::Proc)) {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Incorrect),
        };

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
            Some(s) => s,
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
                match self.expression() {
                    ParserState::Continue => {},
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

                return self.follow_if();
            },
            _ => {},
        };

        match self.check_token(TokenType::Keyword(KeywordType::While), token.clone()) {
            ParserState::Continue => {
                match self.expression() {
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                match self.check(TokenType::Keyword(KeywordType::Do)) {
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                return self.statement();
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
                        let v = match self.last_token() {
                            Some(t) => {
                                // If there's a value then we successfully parsed the Identifier
                                log!("<YASLC/Parser> Parsed PROMPT with identifier, adding to compiled file.");
                                format!("${}", t.lexeme())
                            },
                            None => {
                                // If there's no value, we have no identifier
                                log!("<YASLC/Parser> Parsed PROMPT without identifier, using $junk and adding to compiled file.");
                                "$junk".to_string()
                            }
                        };

                        // Prompt for the variable
                        log!("<YASLC/Parser> Adding prompt command for variable {}", v);

                        self.add_command(&*format!("inw {}", v));

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

        match self.check(TokenType::Assign) {
            ParserState::Continue => {
                return self.expression();
            },
            _ => {},
        };

        self.insert_last_token();

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

            _ => {},
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
        self.expression()
    }

    fn expression(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting EXPRESSION rule.");

        self.add_command("movw SP R1");

        let mut stack = Vec::<Token>::new();

        while self.tokens.is_empty() == false {
            let t = self.tokens.remove(0);
            match self.check_token(TokenType::Semicolon, t.clone()) {
                // If it is a semicolon next token
                ParserState::Continue => {
                    log!("<YASLC/Parser> Exiting EXPRESSION rule because we found the SEMI token.");

                    self.tokens.insert(0, t);
                    return self.parse_expression_tokens(stack);
                },

                // If it is not a semicolon
                _ => {
                    // if it is end
                    match self.check_token(TokenType::Keyword(KeywordType::End), t.clone()) {
                        ParserState::Continue => {
                            log!("<YASLC/Parser> Exiting EXPRESSION rule because we found END token.");

                            self.tokens.insert(0, t);
                            return self.parse_expression_tokens(stack);
                        },
                        _ => {
                            match self.check_token(TokenType::RightParen, t.clone()) {
                                ParserState::Continue => {
                                    // if it is not end but instead right paren
                                    log!("<YASLC/Parser> Exiting EXPRESSION rule because we found RPAREN token.");

                                    self.tokens.insert(0, t);
                                    return self.parse_expression_tokens(stack);
                                },
                                _ => {}
                            };
                        },
                    };

                    stack.push(t);
                }
            };
        }

        log!("<YASLC/Parser> Exiting EXPRESSION rule because unexpectedly we ran out of tokens.");

        ParserState::Done(ParserResult::Unexpected)
    }

    fn parse_expression_tokens(&mut self, tokens: Vec<Token>) -> ParserState {
        match ExpressionParser::new(self.symbol_table.clone(), tokens) {
            Some(e) => {
                log!("<YASLC/Parser> Expression parser successfully exited!");

                // Add the commands to this list of commands
                for c in e.commands() {
                    log!("{}", c);
                    self.add_command(&*c);
                }

                // Reset the symbol table
                self.symbol_table.reset_offset();

                // Set the expression parser to our field and continue
                self.e_parser = Some(e);
                ParserState::Continue
            },
            None => {
                log!("<YASLC/Parser> Expression parser was not successful.");
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
enum ParserResult {
    /// The parser should continue parsing starting with the next token.
    Success,

    /// The parser found the incorrect token but it is not unexpected and should be pushed
    /// back onto the stack and try with a different rule.
    Incorrect,

    // The parser reached an unexpected token, should return an error and stop.
    Unexpected,
}
