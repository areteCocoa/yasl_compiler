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

    pub fn read_endless(&mut self) {
        loop {
            self.read();
        }
    }
    
    pub fn read(&mut self) {
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
    fn handle_line(&mut self, line: String) -> Vec<Token> {
        let chars = line.chars();

        let mut token_state = TokenState::start();
        let mut lexeme = "".to_string();

        let mut tokens = Vec::<Token>::new();
        
        for c in chars {
            token_state = token_state.next_state(c);
            
            if let TokenState::Accept(action, t) = token_state {
                if let TokenAction::Ignore = action {
                    lexeme = String::new();
                    token_state = TokenState::start();
                    continue;
                }
                
                match action {
                    TokenAction::Accept => {
                        lexeme.push(c);
                    },
                    _ => {}
                };

                let mut token_builder = TokenBuilder::new();
                token_builder = token_builder.lexeme(lexeme.clone());
                
                let final_type = match t {
                    TokenType::Identifier => {
                        // See if lexeme is a keyword and change type to keyword if it is
                        if lexeme == "program" || lexeme == "const" || lexeme == "begin" || lexeme == "print"
                            || lexeme == "end" || lexeme == "div" || lexeme == "mod" {
                                TokenType::Keyword
                            } else {
                                TokenType::Identifier
                            }
                        
                    }
                    _ => t,
                };

                token_builder = token_builder.token_type(final_type).column(self.column_number - (lexeme.len() as u32)).line(self.line_number);
                
                //let mut token_builder = TokenBuilder::new();
                //token_builder = token_builder.token_type(t.clone()).lexeme(lexeme.clone());                
                let token = token_builder.token();
                tokens.push(token);

                // Reset token builder variables
                token_state = TokenState::start();
                lexeme = "".to_string();
            } else if let TokenState::Unaccepted = token_state {
                let mut token_builder = TokenBuilder::new();
                token_builder = token_builder.token_type(TokenType::Invalid).lexeme(lexeme.clone()).column(self.column_number).line(self.line_number);
                let token = token_builder.token();
                tokens.push(token);

                token_state = TokenState::start();
                lexeme = "".to_string();
            } else {
                lexeme.push(c);
            }

            // Increment our position variables
            if c == '\n' {
                self.line_number = self.line_number + 1;
                self.column_number = 0;
            } else {
                self.column_number = self.column_number + 1;
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
    CommentSlashStart, // 6
    CommentSlash, // 7
    
    Accept(TokenAction, TokenType),
    Unaccepted,
}

#[derive(Copy, Clone)]
enum TokenAction {
    Accept,
    AcceptPushback,
    Ignore,
}

impl TokenState {
    fn start() -> TokenState {
        TokenState::Start
    }

    fn next_state(self, input: char) -> TokenState {
        match self {
            // Starting state
            TokenState::Start => {
                // Check for ignored characters first
                if input == '\n' || input == ' ' {
                    TokenState::Start
                } else if input.is_alphabetic() {
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
                } else if input == '/' {
                    TokenState::CommentSlashStart
                } else if input == '{' {
                    TokenState::CommentCurly
                }
                else {
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

            TokenState::String => {
                if input == '"' {
                    TokenState::Accept(TokenAction::Accept, TokenType::String)
                } else {
                    TokenState::String
                }
            },
            
            TokenState::CommentCurly => {
                if input == '}' {
                    TokenState::Accept(TokenAction::Ignore, TokenType::Invalid)
                } else {
                    TokenState::CommentCurly
                }
            },

            TokenState::CommentSlashStart => {
                if input == '/' {
                    TokenState::CommentSlash
                } else {
                    TokenState::Unaccepted
                }
            }
            
            TokenState::CommentSlash => {
                if input == '\n' {
                    TokenState::Accept(TokenAction::Ignore, TokenType::Invalid)
                } else {
                    TokenState::CommentSlash
                }
            }
            
            _ => {
                TokenState::Unaccepted
            }

        }
    }
}
