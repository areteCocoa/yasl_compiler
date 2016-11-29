// Thomas Ring
// August 30, 2016
// main.rs
//

static mut VERBOSE: bool = true;

macro_rules! log {
    ($message:expr $(,$arg:expr)*) => {
        unsafe {
            if VERBOSE == true {
                println!($message, $($arg,)*);
            }
        }
    };
}

// include the lib.rs file
extern crate yasl_compiler;

use yasl_compiler::lexer::{LexerResult, LexerError, read_file};

// Include the io lib
use std::io;
use std::env;

fn main() {
    // Check for an argument
    let mut i = 0;
    let mut maybe_file: Option<String> = None;
    for argument in env::args() {
        if i == 0 {
            // Do nothing, its how to program was invoked
        } else {
            // Check for flags
            if argument == "-v" {
                unsafe {
                    VERBOSE = true;
                }
            } else {
                log!("Compiling file \"{}\"", argument);
                maybe_file = Some(argument.clone());
            }
        }

        log!("Argument {}: {}", i, argument);

        i += 1;
    }

    let mut file_name = match maybe_file {
        Some(f) => f,
        None => {
            // Prompt the user for the input
            println!("Please input the name of the YASL file: ");

            // File name from standard input
            let mut input = String::new();

            match io::stdin().read_line(&mut input) {
                Ok(_) => {},
                Err(e) => {
                    println!("<YASLC> Error reading from stdin: {}", e);
                    return;
                },
            };

            input
        }
    };


    //
    // // Get rid of the return character from the end of the string
    // // if it is a newline character
    if let Some(last) = file_name.pop() {
        if last != '\n' {
            file_name.push(last);
        }
    }

    let tokens = match read_file(file_name) {
        LexerResult::Ok(t) => t,
        LexerResult::Err(e) => {
            println!("<YASLC/Lexer> Error reading file. Attempting to find the error...");
            let os_error = std::io::Error::last_os_error();
            println!("This is the last OS error we could find: {}", os_error);
            return;
        }
    };

    log!("<YASLC> Successful lexical analysis of file. Parsing.");

    let mut parser = yasl_compiler::parser::Parser::new_with_tokens(tokens);
    parser.parse();
}
