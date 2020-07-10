use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::error::Error;

const TOKEN_FILE: &str = "./token";

fn get_token_env() -> Result<String, std::env::VarError> {
    std::env::var("DISCORD_TOKEN")
}

fn get_token_file() -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    let token_path = Path::new(TOKEN_FILE);
    File::open(token_path)?.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn get_token() -> Result<String, impl Error> {
    match get_token_env() {
        Ok(tok) => Ok(tok),
        _ => match get_token_file() {
            Ok(tok) => Ok(tok),
            Err(e) => Err(e)
        }
    }
}