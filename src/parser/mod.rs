
use super::lexer::token::{Token, TokenType, KeywordType};

use std::collections::HashMap;

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
            ParserState::Constants(s) => self.parse_assignment(token, s),
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
            AssignmentState::None => {
                // We're not in the middle of an assignment
            },
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

                self.print_constants();
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

    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token.clone());
        self.stack.push(token);
    }

    pub fn print_constants(&self) {
        for (lexeme, value) in self.constants.iter() {
            println!("{}:{}", lexeme, value);
        }
    }
}

#[derive(Clone)]
enum ParserState {
    Start(StartState),
    Constants(AssignmentState),
    Body(BodyState),
}

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

#[derive(Clone)]
enum BodyState {
    Begin,
    Statements,
    End,
}

// enum ParserState {
//     Constant(Option<Token>),
// }
//
// impl ParserState {
//     fn validate_next(self) -> Result<ParserState> {
//
//     }
// }
