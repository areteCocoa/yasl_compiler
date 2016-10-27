// Thomas Ring
// August 30, 2016
// main.rs
//

// include the lib.rs file
extern crate yasl_compiler;

// Include the io lib
use std::io;

fn main() {

    // Create a new lexer (Scanner)
    //let mut scanner = yasl_compiler::lexer::scanner::Scanner::new();

    // Create a new parser
    // let mut parser = yasl_compiler::parser::Parser::new();
    //
    // loop {
    //     scanner.read();
    //
    //     let new_tokens = scanner.new_tokens.clone();
    //     // for t in new_tokens.iter() {
    //     //     println!("{}", t);
    //     // }
    //
    //     parser.parse_line(new_tokens);
    // }

    // File name from standard input
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(e) => {
            panic!("Error reading from stdin: {}", e);
        },
    }
    // Get rid of the return character from the end of the string
    input.pop();

    // File name as an argument
    // Get the last argument, the file name
    // let mut arguments = std::env::args();
    // let file_name = match arguments.nth(1) {
    //     Some(f) => f,
    //     None => {
    //         println!("Please input a file name or the -stdin flag.");
    //         return;
    //     },
    // };

    let scanner = yasl_compiler::lexer::scanner::Scanner::new_from_file(input);
    let tokens = match scanner.read_file() {
        Ok(tokens) => {
            tokens
        }
        Err(e) => {
            println!("Did not successfully read file because {}", e);
            return;
        },
    };

    let mut parser = yasl_compiler::parser::Parser::new_with_tokens(tokens);
    parser.parse();

    // if file_name == "-stdin" {
    //     //let mut scanner = yasl_compiler::lexer::scanner::Scanner::new();
    // } else {
    //     let mut scanner = yasl_compiler::lexer::scanner::Scanner::new_from_file(file_name);
    //     //scanner.read_file();
    // }
}
