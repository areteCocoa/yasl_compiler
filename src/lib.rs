mod lexer;
mod parser;

use lexer::{LexerResult, LexerError};
use lexer::read_file;

pub use parser::{Parser, ParserResult};

pub fn compile_file(file_name: String) -> ParserResult {
    let tokens = match read_file(file_name) {
        LexerResult::Ok(t) => t,
        LexerResult::Err(e) => {
            match e {
                LexerError::FileError => println!("<YASLC> Encountered a file error!"),
                LexerError::StdinError => println!("<YASLC> Encountered an error with stdin!"),
            };

            println!("<YASLC/Lexer> Error reading file. Attempting to find the error...");
            let os_error = std::io::Error::last_os_error();
            println!("This is the last OS error we could find: {}", os_error);
            return ParserResult::Unexpected;
        }
    };

    println!("<YASLC> Successful lexical analysis of file. Parsing.");

    let mut parser = Parser::new_with_tokens(tokens);
    parser.parse()
}
