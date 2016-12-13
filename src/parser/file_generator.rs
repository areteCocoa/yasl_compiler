/// parser/file_generator.rs
///
/// file_generator contains code to generate the final file given a list of commands
/// as well as a list of constant declarations.

use std::fs::File;
use std::io::prelude::*;
use std::io;

/// Generates a file given the list of commands and list of declarations and returns the
/// io::Result, containing Ok(file) if it was successful and Err(e) if it was not.
pub fn file_from(commands: Vec<String>) -> io::Result<File> {
    let mut f = try!(File::create("out.yasl"));

    for c in commands {
        match f.write_fmt(format_args!("{}\n", c)) {
            Ok(_) => {
                //println!("Successfully wrote {:?} bytes to file.", n);
            },
            Err(e) => {
                println!("Error writing to file: {:?}!", e);
            },
        };
    }

    Ok(f)
}
