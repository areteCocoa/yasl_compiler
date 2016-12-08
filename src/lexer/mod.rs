/// lexer/mod.rs
///
/// The lexer module is responsible for lexical analysis of the input file.
///
/// It consists of many other modules which scan the file and handle token
/// generation for the files.


mod scanner;
mod token;

pub use lexer::token::{Token, TokenType, KeywordType};

use lexer::scanner::Scanner;

/// LexerResult is either Ok and includes a vector of the tokens that were
/// returned from the lexical analysis or has an error and returns the
/// appropriate error.
pub enum LexerResult {
    Ok(Vec<Token>),
    Err(LexerError),
}

/// LexerError corresponds to a file error, usually at the OS level.
pub enum LexerError {
    FileError,
    StdinError,
}

/// read_file takes a file name as an input and attempts to do lexical analysis
/// on it using the scanner submodule, then returns the result based on
/// what is returned.
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
