use std::fs::File;
use std::io::prelude::*;
use std::io;

pub fn file_from(commands: Vec<String>, declarations: Vec<String>) -> io::Result<File> {
    let mut f = try!(File::create("out.yasl"));

    for d in declarations {
        match f.write_fmt(format_args!("{}\n", d)) {
            Ok(n) => {},
            Err(e) => {},
        };
    }

    for c in commands {
        match f.write_fmt(format_args!("{}\n", c)) {
            Ok(n) => {},
            Err(e) => {},
        };
    }

    Ok(f)
}
