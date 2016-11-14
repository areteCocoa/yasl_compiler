mod scanner;
mod token;

pub use lexer::token::{Token, TokenType, KeywordType};

use lexer::scanner::Scanner;

pub enum LexerResult {
    Ok(Vec<Token>),
    Err(LexerError),
}

pub enum LexerError {
    FileError,
    StdinError,
}

// Lexer is responsible for lexical analysis for the compiler
// and coordinating the scanner and the token creations.
pub fn read_file(file_name: String) -> LexerResult {
    if let Some(scanner) = Scanner::new_from_file(file_name) {
        match scanner.read_file() {
            Ok(tokens) => {
                return LexerResult::Ok(tokens);
            },
            Err(e) => {
                println!("<YASLC/Lexer> Error reading file: {}", e);
                return LexerResult::Err(LexerError::FileError);
            }
        };
    } else {
        return LexerResult::Err(LexerError::FileError);
    }
}
