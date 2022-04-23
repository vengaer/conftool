use crate::ConfigEntry;
use crate::graph::{state, Graph};
use std::error;

#[derive(Debug)]
pub enum ListOp {
    Show(String),
    All,
    Dependencies(String)
}

pub fn show(option: &str, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
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

pub fn dependencies(option: &str, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;
    let deps = graph.dependencies_of(&option)?;
    println!("{}:", option);
    if deps.len() == 0usize {
        println!("  None");
    }
    else {
        for dep in &deps {
            println!("  {}", dep);
        }
    }

    Ok(())
}
