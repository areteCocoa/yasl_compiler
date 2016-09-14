// scanner.rs
//
// Thomas Ring
// August 30, 2016
//

// Include the token struct and functions
use lexer::token::*;

// Include input methods and string classes
use std::io::{self, Read};

// Define a Scanner struct (class)
pub struct Scanner {
    // Public fields
    //
    //
    // Private (implied) fields
    line_number: u32,
    column_number: u32,
    line: String,
    source: String,
    // file: File
}


impl Scanner {
    // Public methods
    pub fn new() -> Scanner {
        Scanner {
            line_number: 0,
            column_number: 0,
            line: "".to_string(),
            source: "".to_string(),
        }
    }
    
    pub fn next(&self) -> Token {
        // TODO implement the state machine here:
        // - have a "state" variable start in the initial state
        // - repeatedly look at current_char() (the current character),
        //   perform an appropriate action based on them, and assign
        //   a new state until the end of a token is seen
        // - call advance() on each state transition, until you
        //   see the first character after the token
        // - if at_EOF() is true, then return an EOF token:
        //     token.line = line_num;
        //     token.column = column_num;
        //     token.type = EOFILE;
        //     token.lexeme = "";
        if self.at_EOF() {
            
        }

        Token::new()
    }

    pub fn read_endless(&self) {
        loop {
            self.read();
        }
    }
    
    pub fn read(&self) {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                //println!("{} bytes read", n);
                //println!("{}", input);
                let results = self.handle_line(input.clone());
                for result in results {
                    println!("{}", result);
                }
                
            }
            Err(error) => println!("error: {}", error),
        };
    }
    
    // Private methods
    fn current_char(&self) -> char {
        //
        'a'
    }

    fn at_EOF(&self) -> bool {false}

    fn advance(&mut self) {}

    /**
      * handle_line() takes a string and analyzes it for tokens
      *
      *
      */
    fn handle_line(&self, line: String) -> Vec<Token> {
        let chars = line.chars();

        let mut token_state = TokenState::start();
        let mut lexeme = "".to_string();

        let mut tokens = Vec::<Token>::new();
        
        for c in chars {
            token_state = token_state.next_state(c);

            if let TokenState::Accept(action, t) = token_state {
                match action {
                    TokenAction::Accept => {
                        lexeme.push(c);
                    },
                    TokenAction::AcceptPushback => {

                    },
                };
                
                let mut token_builder = TokenBuilder::new();
                token_builder = token_builder.token_type(t.clone()).lexeme(lexeme.clone());
                let token = token_builder.token();
                tokens.push(token);

                // Reset token builder variables
                token_state = TokenState::start();
                lexeme = "".to_string();
            } else if let TokenState::Unaccepted = token_state {
                let mut token_builder = TokenBuilder::new();
                token_builder = token_builder.token_type(TokenType::Invalid).lexeme(lexeme.clone());
                let token = token_builder.token();
                tokens.push(token);

                token_state = TokenState::start();
                lexeme = "".to_string();
            } else {
                lexeme.push(c);
            }
        }

        tokens
    }
}

/*
 * DFA code
 */
enum TokenState {
    Start, // 0

    Identifier, // 1

    Number, // 2

    String, // 3
    StringEnd, // 4

    CommentCurly, // 5
    CommentSlash, // 6

    Accept(TokenAction, TokenType),
    Unaccepted,
}

enum TokenAction {
    Accept,
    AcceptPushback,
}

impl TokenState {
    fn start() -> TokenState {
        TokenState::Start
    }

    fn next_state(self, input: char) -> TokenState {
        match self {
            // Starting state
            TokenState::Start => {
                if input.is_alphabetic() {
                    TokenState::Identifier
                } else if let Some(input_digit) = input.to_digit(10) {
                    if input_digit == 0 {
                        TokenState::Accept(TokenAction::Accept, TokenType::Number)
                    } else {
                        TokenState::Number
                    }
                } else if input == '"' {
                    TokenState::String
                } else if input == '.' || input == ';' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Semicolon)
                } else if input == '+' || input == '-' || input == '*' || input == '=' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Invalid)
                } else {
                    TokenState::Unaccepted
                }
            },

            TokenState::Identifier => {
                if input.is_alphabetic(){
                    TokenState::Identifier
                } else if let Some(input_digit) = input.to_digit(10) {
                    TokenState::Identifier
                } else {
                    TokenState::Accept(TokenAction::AcceptPushback, TokenType::Identifier)
                }
            }

            TokenState::Number => {
                if let Some(input_digit) = input.to_digit(10) {
                    TokenState::Number
                } else {
                    TokenState::Accept(TokenAction::AcceptPushback, TokenType::Number)
                }
            }
            
            _ => {
                TokenState::Unaccepted
            }
        }
    }
}


