
/*
 * Token.rs
 *
 * Thomas Ring
 * August 30, 2016
 * token.rs
 *
 */

use std::fmt;

#[derive(Copy, Clone)]
pub enum TokenType {
    // Identifier
    Identifier,

    // Keyword
    Keyword,

    // Numbers
    Number,

    // String
    String,

    // Punctuation
    Semicolon,
    Plus,
    Minus,
    Star,
    Equals,

    // Misc
    EOFile,

    // Invalids
    Invalid
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenType::Identifier => write!(f, "ID"),
            TokenType::Keyword => {
                write!(f, "KEYWORD")
            },
            TokenType::Number => write!(f, "NUM"),
            TokenType::String => {
                write!(f, "STRING")
            },
            TokenType::Semicolon => write!(f, "SEMI"),
            TokenType::Plus => write!(f, "PLUS"),
            TokenType::Minus => write!(f, "MINUS"),
            TokenType::Star => write!(f, "STAR"),
            TokenType::Equals => write!(f, "EQUALS"),
            TokenType::EOFile => write!(f, "EOF"),
            TokenType::Invalid => write!(f, "Invalid"),
        }
    }
}

pub struct Token {
    pub token_type: TokenType,
    line: u32,
    column: u32,
    lexeme: String
}

impl Token {
    pub fn new() -> Token {
        Token {
            // Just testing, remove this before doing TODO
            line: 0,
            column: 0,
            lexeme: "".to_string(),
            token_type: TokenType::EOFile,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}

pub struct TokenBuilder {
    token_state: TokenState,

    line: u32,
    column: u32,
    lexeme: String,
}

impl TokenBuilder {
    pub fn new(column: u32, line: u32) -> TokenBuilder {
        TokenBuilder {
            line: line,
            column: column,

            lexeme: String::new(),

            token_state: TokenState::Start,
        }
    }

    // Takes a character and pushes it to the lexeme and advances the state,
    // returns true if it reaches a final (accepting or invalid) state
    pub fn push_char(&mut self, c: char) -> (Option<Token>, bool) {
        // Advance the state based on the character
        self.token_state = self.token_state.next_state(c);
        let mut pushback = false;

        match self.token_state {
            TokenState::Start => self.lexeme = String::new(),
            _ => {self.lexeme.push(c)}
        }

        // Check if the state is now at accepted
        // Cases in this block are also responsible for pushing the character
        // onto the lexeme because of how accepting states sometimes require
        // the character not be pushed.
        let token = match self.token_state {
            TokenState::Accept(action, _) => {
                let final_lexeme = match action {
                    TokenAction::AcceptPushback => {
                        pushback = true;
                        self.lexeme.pop();
                        self.lexeme.clone()
                    }
                    _ => {
                        self.lexeme.clone()
                    }
                };


                let result = match action {
                    TokenAction::Ignore => None,
                    _ => {
                        Some(Token {
                           token_type: self.final_type(),
                           line: self.line,
                           column: self.column,
                           // Check if the action requires that we push back
                           lexeme: final_lexeme
                       })
                    }
                };

                result
            },

            // Not an accepting case, we have to push the
            _ => {
                (None)
            }
        };



        // If we created a token we have to reset ourselves and increment values
        (token, pushback)
    }

    fn final_type(&self) -> TokenType {
        match self.token_state {
            TokenState::Accept(_, t) => {
                match t {
                    TokenType::Identifier => {
                        let l = self.lexeme.to_lowercase();
                        if l == "program" || l == "const" || l == "begin" || l == "print"
                            || l == "end" || l == "div" || l == "mod" {
                            TokenType::Keyword
                        } else {
                            TokenType::Identifier
                        }
                    },
                    _ => t
                }
            },
            _ => TokenType::Invalid
        }
    }

    // Consumption setter functions
    pub fn line(mut self, line: u32) -> TokenBuilder {
        self.line = line;
        self
    }

    pub fn column(mut self, column: u32) -> TokenBuilder {
        self.column = column;
        self
    }

    pub fn lexeme(mut self, lexeme: String) -> TokenBuilder {
        self.lexeme = lexeme;
        self
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
    fn next_state(&self, input: char) -> TokenState {
        match *self {
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
                    TokenState::Accept(TokenAction::Accept, match input {
                        '+' => TokenType::Plus,
                        '-' => TokenType::Minus,
                        '*' => TokenType::Star,
                        '=' => TokenType::Equals,
                        _ => TokenType::Invalid
                    })
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
