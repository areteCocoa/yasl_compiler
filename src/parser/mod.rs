///
/// The parser module is responsible for syntax parsing of a set of compiler tokens
/// using an LL(1) parser.
///

pub mod symbol;

use super::lexer::{Token, TokenType, KeywordType};

use std::cmp::Ordering;

use std::fmt;

#[allow(unused_imports)]
use self::symbol::{Symbol, SymbolTable, SymbolType, SymbolValueType};

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

#[allow(dead_code)]

/// The Parser struct can check syntax for a set of tokens for validity.
pub struct Parser {
    tokens: Vec<Token>,

    // The last popped token
    last_token: Option<Token>,

    /// The symbol table associated with this parser
    symbol_table: SymbolTable,

    /// The stack of tokens used with the expression parser
    stack: Vec<Token>,
}

/*
 *  The parser is implemented with some convenience functions for many rules. However,
 *  some rules still have to checked "manually." For any rule that can be accessed from a
 *  rule that can go to empty, you must check the first token to make sure you're in the
 *  correct rule.
 */
impl Parser {
    pub fn new_with_tokens(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,

            last_token: None,

            symbol_table: SymbolTable::empty(),

            stack: Vec::<Token>::new(),
        }
    }

    /**
     * Start parsing the input tokens
     */
    pub fn parse(&mut self) {
        match self.program() {
            ParserState::Done(r) => {
                match r {
                    ParserResult::Success => println!("<YASLC/Parser> Correctly parsed YASL program file."),
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

    // Pops the front token off the stack of tokens and returns it.
    fn next_token(&mut self) -> Token {
        let t = self.tokens.remove(0);

        self.last_token = Some(t.clone());

        t
    }

    /// Returns a copy of the last token popped
    fn last_token(&mut self) -> Option<Token> {
        self.last_token.clone()
    }

    /// Inserts the last token into the token set
    fn insert_last_token(&mut self) {
        if let Some(a) = self.last_token() {
            self.tokens.insert(0, a);
            self.last_token = None;
        } else {
            println!("<YASLC/Parser> Internal warning: Attempted to insert the last token into the parser but there is no last token!");
        }
    }

    // Checks the next token for the token type t and returns the parser state (continue or done)
    // based on the input
    fn check(&mut self, t: TokenType) -> ParserState {
        let token = self.next_token();

        log!("<YASLC/Parser> Checking if token {} is of type {}.", token, t);
        log!("\t\t\t {} tokens left in vector.", self.tokens.len());

        self.check_token(t, token)
    }

    fn check_token(&mut self, t: TokenType, token: Token) -> ParserState {
        match token.is_type(t) {
            true => ParserState::Continue,
            false => ParserState::Done(ParserResult::Unexpected),
        }
    }

    // Checks the token for the first token type t1. If it fails it checks the token for type t2.
    // Returns success if either is the type of token.
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

    /**
     * YASL Context free grammar rules
     */

    /*
     *  PROGRAM rule
     */
    fn program(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PROGRAM rule.");

        match self.check(TokenType::Keyword(KeywordType::Program)) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.check(TokenType::Identifier) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.check(TokenType::Semicolon) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.block() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        }

        match self.check(TokenType::Period) {
            ParserState::Continue => {
                log!("<YASLC/Parser> Exiting Parser because we found the final period.");

                ParserState::Done(ParserResult::Success)
            },
            _ => {
                ParserState::Continue
            },
        }
    }

    // BLOCK rule
    fn block (&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting BLOCK rule.");

        match self.consts() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        }

        match self.vars() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        }

        match self.procs() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        }

        match self.check(TokenType::Keyword(KeywordType::Begin)) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.statements() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        let result = match self.check(TokenType::Keyword(KeywordType::End)) {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        result
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

        match self.check(TokenType::Keyword(KeywordType::Const)) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Incorrect),
        };

        let id = match self.check(TokenType::Identifier) {
            ParserState::Continue => {
                self.last_token().unwrap().lexeme()
            },
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.check(TokenType::Assign) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

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

        match self.check(TokenType::Number) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        self.symbol_table.add(id, SymbolType::Constant(SymbolValueType::Bool));

        match self.check(TokenType::Semicolon) {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Unexpected),
        }
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

        match self.check(TokenType::Keyword(KeywordType::Var)) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Incorrect),
        };

        let id = match self.check(TokenType::Identifier) {
            ParserState::Continue => {self.last_token().unwrap().lexeme()},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.check(TokenType::Colon) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

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

        self.symbol_table.add(id, SymbolType::Variable(t));

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

        match self.param_list() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.check(TokenType::Semicolon) {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.block() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        let r = match self.check(TokenType::Semicolon) {
            ParserState::Continue => ParserState::Continue,
            _ => ParserState::Done(ParserResult::Unexpected),
        };

        self.symbol_table = match self.symbol_table.clone().exit(){
            Some(s) => s,
            None => {
                panic!("A symbol table has been popped where it shouldn't have been and we're in big trouble.");
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

        match self.params() {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.check(TokenType::RightParen) {
            ParserState::Continue => ParserState::Continue,
            _ => return ParserState::Done(ParserResult::Unexpected),
        }
    }

    // PARAMS rule
    fn params(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting PARAMS rule.");

        match self.param() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

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

        match self.check(TokenType::Identifier) {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        match self.check(TokenType::Colon) {
            ParserState::Continue => {},
            ParserState::Done(a) => return ParserState::Done(a),
        };

        match self.token_type() {
            ParserState::Continue => ParserState::Continue,
            ParserState::Done(a) => ParserState::Done(a),
        }
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

        match self.statement() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

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
                    ParserState::Continue => {},
                    _ => return ParserState::Done(ParserResult::Unexpected),
                };

                return self.follow_prompt();
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

        match self.statement_tail() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

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

        match self.expression() {
            ParserState::Continue => {},
            _ => return ParserState::Done(ParserResult::Unexpected),
        };

        self.follow_expression()
    }

    // FOLLOW-PROMPT rule
    fn follow_prompt(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting FOLLOW-PROMPT rule.");

        match self.check(TokenType::Comma) {
            ParserState::Continue => {},
            _ => {
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
                println!("Printing string {}", self.last_token().unwrap().lexeme());
                return ParserState::Continue
            },
            _ => self.insert_last_token(),
        }

        self.expression()
    }

    fn expression(&mut self) -> ParserState {
        log!("<YASLC/Parser> Starting EXPRESSION rule.");

        let mut stack = Vec::<Token>::new();

        while self.tokens.is_empty() == false {
            let t = self.tokens.remove(0);
            match self.check_token(TokenType::Semicolon, t.clone()) {
                ParserState::Continue => {
                    let expression_stack = ExpressionStack::new_from_tokens(stack, self.symbol_table.clone());
                    if expression_stack.is_valid() == true {
                        self.tokens.insert(0, t);
                        return ParserState::Continue;
                    } else {
                        return ParserState::Done(ParserResult::Unexpected);
                    }
                },

                _ => {
                    match self.check_token(TokenType::Keyword(KeywordType::End), t.clone()) {
                        ParserState::Continue => {
                            log!("<YASLC/Parser> Exiting EXPRESSION rule because we found END token.");

                            let expression_stack = ExpressionStack::new_from_tokens(stack, self.symbol_table.clone());
                            if expression_stack.is_valid() == true {
                                self.tokens.insert(0, t);
                                return ParserState::Continue;
                            } else {
                                return ParserState::Done(ParserResult::Unexpected);
                            }
                        },
                        _ => {},
                    };

                    stack.push(t);
                }
            };
        }

        log!("<YASLC/Parser> Exiting EXPRESSION rule because we ran out of tokens.");

        ParserState::Done(ParserResult::Unexpected)
    }


}

enum ParserState {
    // The parser should continue and has the token
    Continue,

    // The parser has finished and is returning the result
    Done(ParserResult)
}

enum ParserResult {
    // The parser should continue parsing starting with the next token
    Success,

    Incorrect,

    // The parser reached an unexpected token, should return an error and stop
    Unexpected,
}

/*
 * Expression parser
 *
 */

#[derive(PartialEq, Clone)]
enum Expression {
    // +, -, etc
    // TokenType is wrapped to store what kind of operator this is
    Operator(TokenType),

    // 2, 5, 7.5, etc
    // The string is stored to have the value of the operand
    Operand(String)
}

impl Expression {
    // Creates a new expression from a token
    fn from_token(t: Token) -> Option<Expression> {
        match t.token_type() {
            TokenType::Number => Some(Expression::Operand(t.lexeme())),

            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Keyword(KeywordType::Div)
            | TokenType::Keyword(KeywordType::Mod) | TokenType::GreaterThan | TokenType::LessThan
            | TokenType::GreaterThanOrEqual | TokenType::LessThanOrEqual | TokenType::EqualTo
            | TokenType::NotEqualTo => Some(Expression::Operator(t.token_type())),


            TokenType::Keyword(KeywordType::Print) => Some(Expression::Operator(t.token_type())),

            TokenType::Identifier => Some(Expression::Operand(t.lexeme())),

            _ => {
                None
            },
        }
    }
}

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Expression) -> Option<Ordering> {
        use self::Expression::*;

        match self {
            // *, div, mod (4)
            &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
            | &Operator(TokenType::Keyword(KeywordType::Mod)) => {
                // * or div or mod
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))|
                    &Operator(TokenType::Keyword(KeywordType::Mod)) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Greater),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Greater),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Greater),

                    &Operand(_) => Some(Ordering::Less),

                    _ => None,
                }
            },

            // +, - (3)
            &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => {
                // + or -
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
                    | &Operator(TokenType::Keyword(KeywordType::Mod))
                    | &Operand(_) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Less),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Greater),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Greater),

                    _ => None,
                }
            },

            // >, <, >=, <= (2)
            &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
            | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual) => {
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
                    | &Operator(TokenType::Keyword(KeywordType::Mod))
                    | &Operand(_) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Less),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Less),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Greater),

                    _ => None,
                }
            },

            &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => {
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
                    | &Operator(TokenType::Keyword(KeywordType::Mod))
                    | &Operand(_) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Less),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Less),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Less),

                    _ => None,
                }
            }

            &Operand(_) => {
                // Any number
                match other {
                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) |
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))|
                    &Operator(TokenType::Keyword(KeywordType::Mod)) => Some(Ordering::Greater),

                    &Operand(_) => None,

                    _ => None,
                }
            }

            _ => None,
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expression::Operator(ref t) => {
                write!(f, "{}", t)
            },
            &Expression::Operand(ref v) => {
                write!(f, "{}", v)
            }
        }
    }
}

// ExpressionStack is responsible for push expressions to the stack as well as
// managing operator precedence for expressions
struct ExpressionStack {
    expressions: Vec<Expression>,

    table: SymbolTable,
}

impl ExpressionStack {
    fn new(table: SymbolTable) -> ExpressionStack {
        ExpressionStack {
            expressions: Vec::<Expression>::new(),
            table: table
        }
    }

    fn new_from_tokens(tokens: Vec<Token>, table: SymbolTable) -> ExpressionStack {
        let mut e_stack = ExpressionStack::new(table);

        for t in tokens.into_iter() {
            if let Some(exp) = Expression::from_token(t) {


                e_stack.push_expression(exp);


            } else {
                log!("<YASLC/ExpParser> Warning: attempted to push invalid token onto expression stack.");
            }
        }

        // while let Some(e) = e_stack.expressions.pop() {
        //     //println!("{}", e);
        // }

        e_stack
    }

    fn push_expression(&mut self, e: Expression) {
        match e.clone() {
            Expression::Operand(o) => {
                log!("<YASLC/ExpParser> Attempting to determine the value of operand...");

                // Check if the token is a symbol or a literal
                match o.parse::<i32>() {
                    Ok(_) => {
                        log!("<YASLC/ExpParser> Detected integer literal.");
                    },
                    Err(_) => {
                        if o == "true" || o == "false" {
                            log!("<YASLC/ExpParser> Detected boolean literal.");
                        } else {
                            // It is a symbol
                            log!("<YASLC/ExpParser> Detected a symbol. Looking up in the symbol table.");
                            // TODO: Set to variable to get value
                            match self.table.get(&*o) {
                                Some(v) => {
                                    match v.symbol_type {
                                        SymbolType::Procedure => panic!("<YASLC/ExpParser> Error: Illicitly used PROC \'{}\' as a variable!"),
                                        _ => {}
                                    }
                                },
                                None => panic!("<YASLC/ExpParser> Error: Used symbol \'{}\' that is not in the symbol table!", o),
                            };
                        }
                    }
                };
            },
            Expression::Operator(_) => {
                // Don't need to determine the value, its an operator
            },
        }

        match e {
            Expression::Operand(_) => {},
            Expression::Operator(_) => {
                // if the stack is empty, just push it
                if self.expressions.len() <= 0 {
                    self.expressions.push(e);
                } // if the item is an operator (this will need to be changed with the addition of parenthesis)
                else {
                    // if the item on the top of the stack is lower priority
                    if e >= self.expressions[self.expressions.len() - 1] {
                        self.expressions.push(e);
                    } else {
                        // Pop items off the stack, write to output, until we get to one with lower
                        // priority (or the stack empties), then push item to stack
                        while e <= self.expressions[self.expressions.len() - 1] && self.expressions.len() > 1 {
                            if let Some(_) = self.expressions.pop() {
                                //println!("{}", x);
                            } else {
                                break;
                            }
                        }
                        self.expressions.push(e);
                    }
                } // else if parenthesis
            }
        }
    }

    fn is_valid(&self) -> bool {
        true
    }

    // TODO: Move to display trait
    // fn print_stack(&self) {
    //     for e in self.expressions.iter() {
    //         println!("{}", e);
    //     }
    // }
}
