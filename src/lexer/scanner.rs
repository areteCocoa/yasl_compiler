// scanner.rs
//
// Thomas Ring
// August 30, 2016
//

// Include the token struct and functions
use lexer::token::*;

// Include input methods and string classes
use std::io::{self, Read};
use std::fs::File;

// Define a Scanner struct (class)
pub struct Scanner {
    // Public fields
    //
    //
    // Properties
    file: File,

    // Used to construct tokens
    // We store the token_builder, which already stores line and column number,
    // in addition to the line and column number because the start line/column
    // of the token does not change but the cursor position does
    token_builder: TokenBuilder,
    line_number: u32,
    column_number: u32,

    // Storing tokens
    pub tokens: Vec<Token>,

    // The set of tokens from the last input
    pub new_tokens: Vec<Token>,
}

impl Scanner {

    // pub fn new() -> Scanner {
    //     // Set a dummy file
    //
    //     // Set the line number and column number
    //     let line_number = 1;
    //     let column_number = 1;
    //     let token_builder = TokenBuilder::new(column_number, line_number);
    //
    //     Scanner {
    //         file: None,
    //         token_builder: token_builder,
    //         line_number: line_number,
    //         column_number: column_number,
    //         tokens: Vec::<Token>::new(),
    //         new_tokens: Vec::<Token>::new(),
    //     }
    // }

    // Creates a new scanner from the file_string and returns it
    pub fn new_from_file(file_string: String) -> Scanner {
        // Open the file so we can set it as a property
        let file = match File::open(file_string.clone()) {
            Ok(f) => f,
            Err(e) => panic!("Error opening file \"{}\": {}", file_string, e),
        };

        // Set the line number and column number
        let line_number = 0;
        let column_number = 0;
        let token_builder = TokenBuilder::new(column_number, line_number);

        Scanner {
            file: file,
            token_builder: token_builder,
            line_number: line_number,
            column_number: column_number,
            tokens: Vec::<Token>::new(),
            new_tokens: Vec::<Token>::new(),
        }
    }

    // Reads the file for this scanner and returns Ok(tokens) where tokens
    // is a list of tokens or Err(error message) where error message is an
    // string describing the error. Consumes the scanner.
    pub fn read_file(mut self) -> Result<Vec<Token>, String> {
        // Read the string to a file
        let mut buffer = String::new();

        // Read the file to the buffer
        match self.file.read_to_string(&mut buffer){
            Ok(size) => {
                // println!("File read of size {}", size);
            },
            Err(e) => {
                // println!(, e);
                return Err(format!("Error reading file to string: {}", e));
            }
        };

        // Input the file one character at a time
        for c in buffer.chars() {
            self.push_char(c);
        }

        Ok(self.tokens)
    }

    // Reads a single line from stdin
    pub fn read(&mut self) {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => self.handle_line(input.clone()),
            Err(e) => println!("Error reading from stdin: {}", e),
        }
    }

    fn handle_line(&mut self, line: String) {
        self.new_tokens = Vec::<Token>::new();
        for c in line.chars() {
            self.push_char(c);
        }
    }

    fn push_char(&mut self, c: char) {
        // Push the char to the builder and get the results (Option<Token>, pushback?)
        let (token, pushback) = self.token_builder.push_char(c);

        // Increment the column and line unless we're going to pushback
        if pushback == false {
            self.increment(c);
        }

        // If we're in the start state, reset the column and line to the current column and line
        if self.token_builder.is_start() {
            self.token_builder.column(self.column_number);
            self.token_builder.line(self.line_number);
        }

        // Check if we got a token and push it to the list of tokens if we do
        if let Some(t) = token {
            self.push_token(t);
            self.token_builder = TokenBuilder::new(self.column_number, self.line_number);
        }

        // If we need to push the cursor back, we just re-read the current character
        if pushback == true {
            self.push_char(c);
        }
    }

    // Increments the line and column states based on the input
    fn increment(&mut self, c: char) {
        if c == '\n' {
            self.column_number = 1;
            self.line_number += 1;
        } else {
            self.column_number += 1;
        }
    }

    // Pushes the token onto the list and prints it
    fn push_token(&mut self, t: Token) {
        // Comment this line to stop printing tokens when they are generated
        let debug = false;
        if debug == true {
            println!("<YASLC/lexer> Generated token: {}", t);
        }


        // Push the token to the vector
        self.new_tokens.push(t.clone());
        self.tokens.push(t);
    }
}
