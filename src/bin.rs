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

    // Create a new parser
    let mut parser = yasl_compiler::parser::Parser::new();

    loop {
        scanner.read();

        let new_tokens = scanner.new_tokens.clone();
        // for t in new_tokens.iter() {
        //     println!("{}", t);
        // }

        parser.parse_line(new_tokens);
    }

    // Get the last argument, the file name
    // let mut arguments = std::env::args();
    // let file_name = match arguments.nth(1) {
    //     Some(f) => f,
    //     None => {
    //         println!("Please input a file name or the -stdin flag.");
    //         return;
    //     },
    // };
    //
    // if file_name == "-stdin" {
    //     let mut scanner = yasl_compiler::lexer::scanner::Scanner::new();
    // } else {
    //     let mut scanner = yasl_compiler::lexer::scanner::Scanner::new_from_file(file_name);
    //     //scanner.read_file();
    // }
}
