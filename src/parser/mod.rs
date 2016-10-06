
use super::lexer::token::{Token, TokenType, KeywordType};

use std::collections::HashMap;
use std::cmp::Ordering;

use std::fmt;

pub struct Parser {
    tokens: Vec<Token>,

    constants: HashMap<String, String>,

    stack: Vec<Token>,

    state: ParserState,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokens: Vec::<Token>::new(),
            constants: HashMap::<String, String>::new(),
            stack: Vec::<Token>::new(),

            state: ParserState::Start(StartState::Start),
        }
    }

    // pub fn parse_file(&mut self, tokens: Vec<Token>) {
    //
    // }

    pub fn parse_line(&mut self, tokens: Vec<Token>) {
        for token in tokens.into_iter() {
            self.parse_token(token);
        }
    }

    pub fn parse_token(&mut self, token: Token) {
        let debug = false;
        if debug == true {
            println!("<YASLC/Parser> Parsing token: {}", token);
        }

        // Push a copy of the token onto our vector of tokens
        self.push_token(token.clone());

        match self.state.clone() {
            ParserState::Start(s) => self.parse_start(token, s),
            ParserState::Constants(s) => {
                match token.token_type() {
                    TokenType::Keyword(KeywordType::Begin) => {
                        // we've ended the constants and are moving to the executed section
                        self.state = ParserState::Body(BodyState::Start);
                        self.parse_body(token, BodyState::Start);
                    }
                    _ => self.parse_assignment(token, s)
                }
            },
            ParserState::Body(s) => self.parse_body(token, s),
        }




        // Otherwise, we have no idea what we're in the middle of and try and figure that
        // out based on the next token
        // match token.token_type() {
        //     TokenType::Keyword => {
        //         // If its a const declaration we can advance assignment state
        //         if token.lexeme().to_lowercase() == "const" {
        //             self.assignment_state = AssignmentState::Keyword;
        //         }
        //     }
        //
        //     _ => {},
        // }
    }

    fn parse_start(&mut self, token: Token, state: StartState) {
        // Advance the start state
        let new_state = state.next(token.token_type());

        // Check if the new state is either finished or invalid
        match new_state {
            StartState::Semicolon => {
                // In the future, we may need to store the variables like the program
                // name or even check them. We'll use the stack for this.
                self.stack.clear();

                self.state = ParserState::Constants(AssignmentState::None);
            },
            StartState::Invalid(err_string) => {
                println!("<YASLC/Parser> ({}, {}) {}", token.line(), token.column(), err_string);
            },
            _ => {
                self.state = ParserState::Start(new_state);
            },
        };
    }

    fn parse_assignment(&mut self, token: Token, state: AssignmentState) {
        // Advance the assignment state
        let new_state = state.next(token.token_type());

        // Check to see where we are now.
        match new_state {
            AssignmentState::Semicolon => {
                // Finished the assignment, get the values and store them
                let mut column = 0;
                let mut line = 0;
                let mut lexeme = String::new();
                let mut value = String::new();

                // Pop the stack and find the value and lexeme
                while let Some(t) = self.stack.pop() {
                    match t.token_type() {
                        TokenType::Keyword(_) => {
                            column = t.column();
                            line = t.line();
                        }
                        TokenType::Identifier => lexeme = t.lexeme(),
                        TokenType::Number => value = t.lexeme(),
                        _ => {},
                    };
                }

                // Check that we don't already have a value for this lexeme
                if let Some(_) = self.constants.get(&lexeme) {
                    // We already have a value, print an error
                    println!("<YASLC/Parser> ({}, {}) Unexpected reassignment of a constant.",
                    line, column);
                } else {
                    // Push the lexeme and value to the hashmap
                    self.constants.insert(lexeme, value);
                }

                // Reset the assignment state
                self.state = ParserState::Constants(AssignmentState::None);

                //self.print_constants();
            }

            AssignmentState::Invalid(ref err_string) => {
                // We had an unexpected error parsing the token
                println!("<YASLC/Parser> ({}:{}) Unexpected token: {}", token.line(), token.column(),
                err_string);
                self.state = ParserState::Constants(AssignmentState::None);
            },
            _ => {
                self.state = ParserState::Constants(new_state);
            },
        }
    }

    fn parse_body(&mut self, token: Token, state: BodyState) {
        // Advance to the next state
        let new_state = state.next(token.token_type());

        // enum BodyState {
        //     // We've entered the body state
        //     Start,
        //
        //     // Refering to the keyword begin
        //     Begin,
        //     Statements,
        //     End,
        //
        //     Invalid(String)
        // }
        match new_state {
            BodyState::End => {
                // End
                // We don't need to do anything YET
                // TODO: do something
            },
            BodyState::Invalid(s) => {
                println!("<YASLC/Parser> Unexpected error with parser: {}", s);
            },
            BodyState::Statements => {
                // Check the statements state
                match token.token_type() {
                    TokenType::Semicolon => {
                        let e_stack = ExpressionStack::new_from_tokens(self.pop_stack());
                        e_stack.print_stack();
                    },
                    _ => {}, // We're still in the middle of an expression
                }
            }
            _ => {
                self.state = ParserState::Body(new_state);
            },
        }


    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token.clone());

        // Push to the stack but check if its a constant
        if let Some(value) = self.constants.get(&token.lexeme()) {
            let mut value_token = token.clone();
            value_token.set_lexeme(value.clone());
            value_token.set_token_type(TokenType::Number);
            self.stack.push(value_token);
        } else {
            self.stack.push(token);
        }
    }

    pub fn print_constants(&self) {
        for (lexeme, value) in self.constants.iter() {
            println!("{}:{}", lexeme, value);
        }
    }

    fn pop_stack(&mut self) -> Vec<Token> {
        let stack = self.stack.clone();

        self.stack.clear();

        return stack;
    }
}

// ParserState is the state of the parser relative to the input it has received
#[derive(Clone)]
enum ParserState {
    Start(StartState),
    Constants(AssignmentState),
    Body(BodyState),
}

// StartState is the state of the parser in the start of the parsed input
#[derive(Clone)]
enum StartState {
    Start,
    Program,
    ProgramName,
    Semicolon,
    Invalid(String),
}

impl StartState {
    fn next(&self, token_type: TokenType) -> StartState {
        match (self.clone(), token_type) {
            (StartState::Start, TokenType::Keyword(KeywordType::Program)) => StartState::Program,
            (StartState::Program, TokenType::Identifier) => StartState::ProgramName,
            (StartState::ProgramName, TokenType::Semicolon) => StartState::Semicolon,
            _ => {
                // TODO: Find out what went wrong and return that error

                StartState::Invalid("Unexpected token.".to_string())
            },
        }
    }
}

// AssignmentState is the state of the parser in the header const/var assignments
#[derive(Clone)]
enum AssignmentState {
    None,
    Keyword,
    Identifier,
    Equals,
    Value,
    Semicolon,
    Invalid(String),
}

impl AssignmentState {
    fn next(&self, token_type: TokenType) -> AssignmentState {
        match (self.clone(), token_type) {
            (AssignmentState::None, TokenType::Keyword(KeywordType::Const)) => AssignmentState::Keyword,
            (AssignmentState::Keyword, TokenType::Identifier) => AssignmentState::Identifier,
            (AssignmentState::Identifier, TokenType::Equals) => AssignmentState::Equals,
            (AssignmentState::Equals, TokenType::Number) => AssignmentState::Value,
            (AssignmentState::Value, TokenType::Semicolon) => AssignmentState::Semicolon,
            _ => {
                // TODO: Find out what caused the error

                AssignmentState::Invalid("Unexpected input for constant assignment.".to_string())
            },
        }
    }
}

// BodyState is the state of the parser in the Body of the input
#[derive(Clone)]
enum BodyState {
    // We've entered the body state
    Start,

    // Refering to the keyword "begin"
    Begin,
    Statements,
    End,

    Invalid(String)
}

impl BodyState {
    fn next(&self, token_type: TokenType) -> BodyState {
        match (self.clone(), token_type) {
            (BodyState::Start, TokenType::Keyword(KeywordType::Begin)) => {
                BodyState::Begin
            },
            (BodyState::Begin, _) => {
                BodyState::Statements
            },
            (BodyState::Statements, TokenType::Keyword(KeywordType::End)) => BodyState::End,
            (BodyState::Statements, _) => BodyState::Statements,
            _ => {
                // TODO: Figure out what got us here

                BodyState::Invalid("Unexpected token.".to_string())
            }
        }
    }
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
    fn from_token(t: Token) -> Option<Expression> {
        match t.token_type() {
            TokenType::Number => Some(Expression::Operand(t.lexeme())),

            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Keyword(KeywordType::Div)
            | TokenType::Keyword(KeywordType::Mod) => Some(Expression::Operator(t.token_type())),

            TokenType::Keyword(KeywordType::Print) => Some(Expression::Operator(t.token_type())),

            _ => None,
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
                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Greater),

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
                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) |
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))|
                    &Operator(TokenType::Keyword(KeywordType::Mod)) => Some(Ordering::Greater),

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
        let mut stack = Vec::<Expression>::new();

        for t in tokens.into_iter() {
            if let Some(exp) = Expression::from_token(t) {
                stack.push(exp);
            } else {

            }
        }

        ExpressionStack {
            expressions: stack
        }
    }

    fn print_stack(&self) {
        for e in self.expressions.iter() {
            println!("{}", e);
        }
    }
}
