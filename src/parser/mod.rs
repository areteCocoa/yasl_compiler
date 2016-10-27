///
/// The parser module is responsible for syntax parsing of a set of compiler tokens
/// using an LL(1) parser.
///


use super::lexer::token::{Token, TokenType, KeywordType};

use std::collections::HashMap;
use std::cmp::Ordering;

use std::fmt;

/// The Parser struct can check syntax for a set of tokens for validity.
pub struct Parser {
    tokens: Vec<Token>,

    constants: HashMap<String, String>,

    stack: Vec<Token>,
}

/*
 *  The parser is implemented with some convenience functions for many rules. However,
 *  some rules still have to checked "manually." For any rule that can be accessed from a
 *  rule that can go to empty, you must check the first token to make sure you're in the
 *  correct rule.
 */
impl Parser {
    // pub fn new() -> Parser {
    //     Parser {
    //         tokens: Vec::<Token>::new(),
    //         constants: HashMap::<String, String>::new(),
    //         stack: Vec::<Token>::new(),
    //
    //         state: ParserState::Start(StartState::Start),
    //     }
    // }

    pub fn new_with_tokens(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            constants: HashMap::<String, String>::new(),
            stack: Vec::<Token>::new(),
        }
    }

    /**
     * Start parsing the input tokens
     */
    pub fn parse(&mut self) {
        match self.program() {
            ParseResult::Success => println!("Correctly parsed YASL program file."),

            _ => {},
        }
    }

    fn next_token(&mut self) -> Token {
        if self.tokens.len() == 0 {
            panic!("Attempted to remove a token for parsing but there are none left!");
        }
        self.tokens.remove(0)
    }

    fn print_error(&self, expected: Option<TokenType>, found: &Token) {
        if let Some(e) = expected {
            panic!("<YASLC/Parser> ({}, {}) Error: Expected token {}, found {}.",
                found.line(), found.column(), e, found);
        } else {
            panic!("<YASLC/Parser> ({}, {}) Error: Unexpected token.",
                found.line(), found.column());
        }

    }

    fn check(&mut self, t: TokenType, token: Token, expected: bool) -> ParseResult {
        if token.is_type(t.clone()) == false {
            if expected == true {
                self.print_error(Some(t), &token);
                return ParseResult::Unexpected(token);
            } else {
                return ParseResult::Incorrect(token);
            }
        }

        ParseResult::Success
    }

    fn check_next(&mut self, t: TokenType, expected: bool) -> ParseResult {
        let token = self.next_token();
        self.check(t, token, expected)
    }

    fn check_next_for(&mut self, types: Vec<TokenType>, expected: bool) -> ParseResult {
        let mut e = expected;
        for t in types.into_iter() {
            match self.check_next(t, e) {
                ParseResult::Unexpected(t) => return ParseResult::Unexpected(t),
                ParseResult::Incorrect(t) => return ParseResult::Incorrect(t),
                _ => {},
            }
            e = true;
        }
        ParseResult::Success
    }

    // Takes a list of parse results and returns a single parse result. If it is unexpected
    // or incorrect it will return the first unexpected or incorrect wrapped token.
    //
    // WARNING: If you are using a method that is combining rules for a grammar that may return
    // empty you will throw away too many tokens. Always have one independent check from the
    // result combination to prevent this.
    fn combine_results(results: Vec<ParseResult>) -> ParseResult {
        for r in results {
            match r {
                ParseResult::Unexpected(t) => return ParseResult::Unexpected(t),
                ParseResult::Incorrect(t) => return ParseResult::Incorrect(t),
                _ => {},
            }
        }
        ParseResult::Success
    }

    /**
     * Context free grammar rules
     */
    fn program(&mut self) -> ParseResult {
        Parser::combine_results(vec![
            self.check_next_for(vec![
                TokenType::Keyword(KeywordType::Program),
                TokenType::Identifier,
                TokenType::Semicolon,
                ], true),
            self.block(),
            self.check_next(TokenType::Period, true)
        ])
    }

    fn block(&mut self) -> ParseResult {
        Parser::combine_results(vec![
            self.consts(),
            self.vars(),
            self.procs(),
            self.check_next(TokenType::Keyword(KeywordType::Begin), true),
            self.statements(),
            self.check_next(TokenType::Keyword(KeywordType::End), true),
        ])
    }

    fn consts(&mut self) -> ParseResult {
        match self.token_const() {
            ParseResult::Unexpected(t) => ParseResult::Unexpected(t),
            ParseResult::Incorrect(t) => {
                self.tokens.insert(0, t);
                ParseResult::Success
            }
            _ => self.consts(),
        }
    }

    fn token_const(&mut self) -> ParseResult {
        // Manually check first rule
        match self.check_next(TokenType::Keyword(KeywordType::Const), false) {
            ParseResult::Unexpected(t) => return ParseResult::Unexpected(t),
            ParseResult::Incorrect(t) => return ParseResult::Incorrect(t),
            _ => {},
        };

        self.check_next_for(vec![
            TokenType::Identifier,
            TokenType::Equals,
            TokenType::Number,
            TokenType::Semicolon
        ], true)
    }

    fn vars(&mut self) -> ParseResult {
        match self.var() {
            ParseResult::Unexpected(t) => ParseResult::Unexpected(t),
            ParseResult::Incorrect(t) => {
                self.tokens.insert(0, t);
                ParseResult::Success
            }
            _ => self.vars(),
        }
    }

    fn var(&mut self) -> ParseResult {
        match self.check_next(TokenType::Keyword(KeywordType::Var), false) {
            ParseResult::Unexpected(t) => return ParseResult::Unexpected(t),
            ParseResult::Incorrect(t) => return ParseResult::Incorrect(t),
            _ => {},
        };

        Parser::combine_results(vec![
            self.check_next_for(vec![
                TokenType::Identifier,
                TokenType::Colon,
            ], true),
            self.token_type(),
            self.check_next(TokenType::Semicolon, true)
        ])
    }

    fn token_type(&mut self) -> ParseResult {
        let token = self.next_token();

        match self.check(TokenType::Keyword(KeywordType::Int), token.clone(), false) {
            ParseResult::Success => return ParseResult::Success,
            _ => {},
        };

        match self.check(TokenType::Keyword(KeywordType::Bool), token.clone(), true) {
            ParseResult::Success => return ParseResult::Success,
            _ => {},
        };

        ParseResult::Unexpected(token)
    }

    fn procs(&mut self) -> ParseResult {
        match self.token_proc() {
            ParseResult::Unexpected(t) => ParseResult::Unexpected(t),
            ParseResult::Incorrect(t) => {
                self.tokens.insert(0, t);
                ParseResult::Success
            }
            _ => self.procs(),
        }
    }

    fn token_proc(&mut self) -> ParseResult {
        match self.check_next(TokenType::Keyword(KeywordType::Proc), false) {
            ParseResult::Unexpected(t) => return ParseResult::Unexpected(t),
            ParseResult::Incorrect(t) => return ParseResult::Incorrect(t),
            _ => {},
        };

        Parser::combine_results(vec![
            self.check_next_for(vec![
                TokenType::Identifier,
                TokenType::Colon
            ], true),
            self.param_list(),
            self.check_next(TokenType::Semicolon, true),
            self.block(),
            self.check_next(TokenType::Semicolon, true),
        ])
    }

    fn param_list(&mut self) -> ParseResult {
        Parser::combine_results(vec![
            self.check_next(TokenType::LeftParen, false),
            self.params(),
            self.check_next(TokenType::RightParen, true),
        ])
    }

    fn params(&mut self) -> ParseResult {
        Parser::combine_results(vec![
            self.param(),
            self.follow_params(),
        ])
    }

    fn follow_params(&mut self) -> ParseResult {
        Parser::combine_results(vec![
            self.check_next(TokenType::Comma, false),
            self.params(),
        ])
    }

    fn param(&mut self) -> ParseResult {
        Parser::combine_results(vec![
            self.check_next_for(vec![
                TokenType::Identifier,
                TokenType::Colon,
            ], true),
            self.token_type()
        ])
    }

    fn statements(&mut self) -> ParseResult {
        match self.statement() {
            ParseResult::Unexpected(t) => return ParseResult::Unexpected(t),
            _ => {},
        };

        match self.statement_tail() {
            ParseResult::Incorrect(t) => {
                if t.is_type(TokenType::Keyword(KeywordType::End)) {
                    self.tokens.insert(0, t);
                    return ParseResult::Success;
                }
                self.print_error(None, &t);
                ParseResult::Unexpected(t)
            },
            ParseResult::Unexpected(t) => {
                return ParseResult::Unexpected(t);
            }
            ParseResult::Success => ParseResult::Success,
        }
    }

    fn statement_tail(&mut self) -> ParseResult {
        // Check the first one manually
        match self.check_next(TokenType::Semicolon, false) {
            ParseResult::Unexpected(t) => {
                return ParseResult::Unexpected(t);
            }
            ParseResult::Incorrect(t) => {
                return ParseResult::Incorrect(t);
            }
            _ => {},
        };

        match self.statement() {
            ParseResult::Unexpected(t) => {
                return ParseResult::Unexpected(t);
            }
            _ => {},
        };

        self.statement_tail()
    }

    fn statement(&mut self) -> ParseResult {
        let token = self.next_token();

        match self.check(TokenType::Keyword(KeywordType::If), token.clone(), false) {
            ParseResult::Success => {
                return Parser::combine_results(vec![
                    self.expression(),
                    self.check_next(TokenType::Keyword(KeywordType::Then), true),
                    self.statement(),
                    self.follow_if(),
                ]);
            },
            _ => {},
        };

        match self.check(TokenType::Keyword(KeywordType::While), token.clone(), false) {
            ParseResult::Success => {
                return Parser::combine_results(vec![
                    self.expression(),
                    self.check_next(TokenType::Keyword(KeywordType::Do), true),
                    self.statement()
                ])
            },
            _ => {},
        };

        match self.check(TokenType::Keyword(KeywordType::Begin), token.clone(), false) {
            ParseResult::Success => {
                return self.follow_begin();
            },
            _ => {},
        };

        match self.check(TokenType::Identifier, token.clone(), false) {
            ParseResult::Success => {
                return self.follow_id();
            },
            _ => {},
        };

        match self.check(TokenType::Keyword(KeywordType::Prompt), token.clone(), false) {
            ParseResult::Success => {
                return Parser::combine_results(vec![
                    self.check_next(TokenType::String, true),
                    self.follow_prompt(),
                ]);
            },
            _ => {},
        };

        match self.check(TokenType::Keyword(KeywordType::Print), token.clone(), false) {
            ParseResult::Success => {
                return self.follow_print();
            },
            _ => {},
        };

        ParseResult::Unexpected(token)
    }

    fn follow_if(&mut self) -> ParseResult {
        match self.check_next(TokenType::Keyword(KeywordType::Else), false) {
            ParseResult::Incorrect(t) => {
                self.tokens.insert(0, t);
                ParseResult::Success
            },
            ParseResult::Success => {
                self.statement()
            },
            ParseResult::Unexpected(t) => {
                ParseResult::Unexpected(t)
            }
        }
    }

    fn follow_id(&mut self) -> ParseResult {
        match self.check_next(TokenType::Equals, false) {
            ParseResult::Incorrect(t) => {
                self.tokens.insert(0, t);
            },
            ParseResult::Success => {
                return self.expression();
            },
            _ => {},
        };

        match self.check_next(TokenType::LeftParen, false) {
            ParseResult::Incorrect(t) => {
                self.tokens.insert(0, t);
            },
            ParseResult::Success => {
                return Parser::combine_results(vec![
                    self.expression(),
                    self.follow_expression(),
                    self.check_next(TokenType::RightParen, true),
                ]);
            },
            _ => {},
        }

        ParseResult::Success
    }

    fn follow_expression(&mut self) -> ParseResult {
        match self.check_next(TokenType::Comma, false) {
            ParseResult::Incorrect(t) => {
                self.tokens.insert(0, t);
                return ParseResult::Success;
            },
            _ => {},
        };

        match self.expression() {
            ParseResult::Unexpected(t) => return ParseResult::Unexpected(t),
            _ => {},
        };

        return self.follow_expression();
    }

    fn follow_begin(&mut self) -> ParseResult {
        match self.statement() {
            ParseResult::Unexpected(t) => {
                self.tokens.insert(0, t);
            },
            ParseResult::Success => {
                match self.statement_tail() {
                    ParseResult::Incorrect(t) => {
                        self.tokens.insert(0, t);
                        return ParseResult::Success;
                    },
                    ParseResult::Success => {
                        return self.check_next(TokenType::Keyword(KeywordType::End), true);
                    },
                    _ => {},
                };

                return Parser::combine_results(vec![
                    self.statement_tail(),

                ]);
            },
            _ => {},
        };

        return self.check_next(TokenType::Keyword(KeywordType::End), true);
    }

    fn expression(&mut self) -> ParseResult {
        let mut stack = Vec::<Token>::new();

        while self.tokens.is_empty() == false {
            let t = self.tokens.remove(0);
            match self.check(TokenType::Semicolon, t.clone(), false) {
                ParseResult::Incorrect(t) => {
                    // Push to stack
                    stack.push(t);
                },
                ParseResult::Success => {
                    let expression_stack = ExpressionStack::new_from_tokens(stack);
                    if expression_stack.is_valid() == true {
                        self.tokens.insert(0, t);
                        return ParseResult::Success;
                    } else {
                        return ParseResult::Unexpected(t);
                    }
                },
                _ => {},
            };
        }

        return ParseResult::Unexpected(stack.pop().unwrap());
    }

    fn follow_prompt(&mut self) -> ParseResult {
        self.check_next_for(vec![
            TokenType::Comma,
            TokenType::Identifier,
        ], true)
    }

    fn follow_print(&mut self) -> ParseResult {
        match self.check_next(TokenType::String, false) {
            ParseResult::Success => return ParseResult::Success,
            ParseResult::Incorrect(t) => {
                self.stack.insert(0, t);
            },
            _ => {},
        };
        self.expression()
    }
}

enum ParseResult {
    // The parser should continue parsing starting with the next token
    Success,

    // The parser should retry the returned token with a different rule
    Incorrect(Token),

    // The parser reached an unexpected token, should return an error and stop
    Unexpected(Token),
}

#[derive(PartialEq)]
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
            | TokenType::Keyword(KeywordType::Mod) => Some(Expression::Operator(t.token_type())),

            TokenType::Keyword(KeywordType::Print) => Some(Expression::Operator(t.token_type())),

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
            &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => {
                // + or -
                match other {
                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Less),

                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
                    | &Operator(TokenType::Keyword(KeywordType::Mod))
                    | &Operand(_) => Some(Ordering::Less),

                    _ => None,
                }
            },

            &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
            | &Operator(TokenType::Keyword(KeywordType::Mod)) => {
                // * or div or mod
                match other {
                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Greater),

                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))|
                    &Operator(TokenType::Keyword(KeywordType::Mod)) => Some(Ordering::Less),

                    &Operand(_) => Some(Ordering::Less),

                    _ => None,
                }
            },

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
}

impl ExpressionStack {
    fn new() -> ExpressionStack {
        ExpressionStack {
            expressions: Vec::<Expression>::new(),
        }
    }

    fn new_from_tokens(tokens: Vec<Token>) -> ExpressionStack {
        let mut e_stack = ExpressionStack::new();

        for t in tokens.into_iter() {
            if let Some(exp) = Expression::from_token(t) {
                e_stack.push_expression(exp);
            } else {

            }
        }

        while let Some(e) = e_stack.expressions.pop() {
            //println!("{}", e);
        }

        e_stack
    }

    fn push_expression(&mut self, e: Expression) {
        match e {
            Expression::Operand(l) => {},
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
                            if let Some(x) = self.expressions.pop() {
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

    fn print_stack(&self) {
        for e in self.expressions.iter() {
            println!("{}", e);
        }
    }
}
