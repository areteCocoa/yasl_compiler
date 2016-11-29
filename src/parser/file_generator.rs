use std::fs::File;

fn file_from_commands(commands: Vec<String>) -> FileGeneratorResult {
    Err("".to_string())
}

enum FileGeneratorResult {
    Ok(File),
    Err(String),
}
