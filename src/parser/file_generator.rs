use std::fs::File;
use std::io::prelude::*;
use std::io;

pub fn file_from_commands(commands: Vec<String>) -> io::Result<File> {
    let mut f = try!(File::create("out.yasl"));

    for c in commands {
        match f.write_fmt(format_args!("{}\n", c)) {
            Ok(n) => {},
            Err(e) => {},
        };
    }

    Ok(f)
}
