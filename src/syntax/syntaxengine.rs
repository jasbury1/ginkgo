use serde_yaml;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::File;
use std::{fs, io};
use std::io::BufReader;

const SYNTAX_DEF_PATH: &str = "definitions/";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum HLMode {
    Normal,
    Symbol,
    Number,
    String,
    Comment,
    Type,
    Call,
    Keyword,
    Special,
    Reserved
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SMatch {
    pattern: String,
    match_type: HLMode
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SyntaxDefinition {
    name: String,
    extensions: Vec<String>,
    contexts: HashMap<String, Vec<SMatch>>
}

struct Highlighter {
    
}

fn read_syntax_file(filename: &str) -> io::Result<SyntaxDefinition> {
    let file = File::open("c_syntax.yaml")?;
    let reader = BufReader::new(file);
    let syntax: SyntaxDefinition = serde_yaml::from_reader(reader).unwrap();
    Ok(syntax)
}
/*
let file = File::open("c_syntax.yaml").unwrap();
    let reader = BufReader::new(file);

    let res: SyntaxDefinition = serde_yaml::from_reader(reader).unwrap();
    */