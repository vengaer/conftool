use crate::ConfigEntry;
use crate::graph::Graph;
use regex::Regex;
use std::{error,fmt,fs,path};


fn validate_line_format<T>(lines: &[T]) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Display + ToString + fmt::Debug
{
    let re = Regex::new(r"^\s*([A-Za-z0-9_-]+\s*=.*)?$").unwrap();
    let mut lineno = 0u32;
    let mut valid = true;
    for line in lines {
        lineno += 1;
        if !re.is_match(&line.to_string()) {
            eprintln!("Syntax error on line {}: {}", lineno, line);
            valid = false;
        }
    }

    if !valid {
        return Err("Errors encountered while parsing config file".into());
    }

    Ok(())
}

pub fn validate_config(path: &path::PathBuf, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let mut graph: Graph<&str> = Graph::new();
    let lines: Vec<String> = fs::read_to_string(path)?
                                .split("\n")
                                .map(|s| s.to_owned())
                                .collect();

    validate_line_format(&lines)?;
    for ent in entries {
        graph.insert(&ent.name, &ent.depends
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>())?;
    }

    Ok(())
}
