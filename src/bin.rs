// Thomas Ring
// August 30, 2016
// main.rs
//

// include the lib.rs file
extern crate yasl_compiler;

fn main() {
    // Use the lexer to go through the input file (until EOF)
    // and identify tokens
    // NOTE Project1: Temporarily reads from standard input (stdin) and prints
    // the tokens

    // Create a new lexer (Scanner)
    let mut scanner = yasl_compiler::lexer::scanner::Scanner::new();
    scanner.read_endless();

    // println!("Hello World!");
}
