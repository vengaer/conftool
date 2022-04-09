use crate::ConfigEntry;
use std::error::Error;

#[derive(Debug)]
pub enum ListOp {
    Show(String),
    All
}

pub fn show(option: &str, entries: &Vec<ConfigEntry>) -> Result<(), Box<dyn Error>> {
    let entry = entries.iter().find(|&ent| ent.name == option);
    match entry {
        Some(entry) => {
            println!("{}", entry);
            Ok(())
        },
        None => Err(format!("Invalid config option {}", option).into())
    }
}

pub fn show_all(entries: &Vec<ConfigEntry>) {
    for ent in entries {
        println!("{}", ent);
    }
}
