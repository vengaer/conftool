#![warn(
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]

pub use crate::list::ListOp;
use std::path::PathBuf;
use std::fmt;
use serde::{Deserialize};

/// Command line management
pub mod cli;
/// Functions related to list subcommand
pub mod list;
/// Json and config parsing
pub mod parser;

#[derive(Debug)]
pub struct State {
    /// Path to config specification
    pub spec: PathBuf,

    /// Path to config file
    pub config: PathBuf,

    /// Verbosity level
    pub verbosity: usize,

    /// Mode of operation
    pub mode: Mode
}

#[derive(Debug)]
pub enum Mode {
    List {
        op: ListOp
    }
}

#[derive(Debug, Deserialize)]
pub enum EntryType {
    Switch(Switch),
    String(String),
    Int(i32)
}

#[derive(Debug)]
pub struct ConfigEntry {
    pub name: String,
    pub depends: Vec<String>,
    pub enttype: EntryType,
    pub choices: Option<Vec<String>>,
    pub help: String
}

#[derive(Debug, Deserialize)]
pub enum Switch {
    Yes,
    No
}

impl fmt::Display for ConfigEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:\n", self.name)?;
        write!(f, "  depends: {:?}\n", self.depends)?;
        let enttype = match &self.enttype {
            EntryType::Switch(_) => "switch",
            EntryType::String(_) => "string",
            EntryType::Int(_) => "integer"
        };

        write!(f, "  type: {}\n", enttype)?;
        let choices = match &self.choices {
            Some(choices) => {
                format!("{:?}", choices)
            },
            None => match &self.enttype {
                EntryType::Switch(_) => "y, n".to_string(),
                _ => format!("Any {}", enttype)
            }
        };
        write!(f, "  choices: {}\n", choices)?;
        write!(f, "  help: {}", self.help)?;
        Ok(())
    }
}
