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
                print!("\n");
            }
            Err(error) => println!("error: {}", error),
        };
    }

    fn handle_line(&mut self, line: String) -> Vec<Token> {
        let mut tokens = Vec::<Token>::new();
        let mut token_builder = TokenBuilder::new(self.column_number, self.line_number);

        for c in line.chars() {
            // Increment line and column
            if c == '\n' {
                self.column_number = 0;
                self.line_number += 1;
            } else {
                self.column_number += 1;
            }

            let results = token_builder.push_char(c);

            token_builder = results.0;
            let token = results.1;
            let pushback = results.2;

            match token {
                Some(t) => {
                    println!("TOKEN");
                    tokens.push(t);
                    token_builder = TokenBuilder::new(self.column_number, self.line_number)
                },
                None => {}
            }
        }

        tokens
    }
}
