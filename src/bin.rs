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

    // Prompt the user for the input
    println!("Please input the name of the YASL file: ");

    // File name from standard input
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(e) => {
            panic!("Error reading from stdin: {}", e);
        },
    }

    // Get rid of the return character from the end of the string
    // if it is a newline character
    if let Some(last) = input.pop() {
        if last != '\n' {
            input.push(last);
        }
    }

    let scanner = yasl_compiler::lexer::scanner::Scanner::new_from_file(input);
    let tokens = match scanner.read_file() {
        Ok(tokens) => {
            tokens
        }
        Err(e) => {
            println!("Did not successfully read file because {}.\nAttempting to find the error...", e);
            let os_error = std::io::Error::last_os_error();
            println!("This is the last OS error we could find: {}", os_error);
            return;
        },
    };

    let mut parser = yasl_compiler::parser::Parser::new_with_tokens(tokens);
    parser.parse();
}
