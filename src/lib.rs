#![warn(
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]

pub use crate::list::ListOp;
use std::path;
use std::fmt;
use serde;

/// Command line management
pub mod cli;
/// Dependency graph
pub mod graph;
/// Functions related to list subcommand
pub mod list;
/// Json and config parsing
pub mod parse;
/// Validation of existing config
pub mod validate;
/// Config manipulation
pub mod manipulate;

#[derive(Debug)]
pub struct State {
    /// Path to config specification
    pub spec: path::PathBuf,

    /// Path to config file
    pub config: path::PathBuf,

    /// Verbosity level
    pub verbosity: usize,

    /// Mode of operation
    pub mode: Mode
}

#[derive(Debug)]
pub enum Mode {
    List {
        ops: Vec<ListOp>
    },
    Validate,
    Enable {
        option: String
    }
}

#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
pub enum EntryType {
    Switch(Switch),
    String(String),
    Int(i32)
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConfigEntry {
    pub name: String,
    pub depends: Vec<String>,
    pub enttype: EntryType,
    pub choices: Option<Vec<String>>,
    pub help: String
}

#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
pub enum Switch {
    Yes,
    No
}

fn format_vec<T>(f: &mut fmt::Formatter<'_>, prefix: Option<&str>, v: &Vec<T>) -> fmt::Result
where
    T: fmt::Display
{
    if let Some(prefix) = prefix {
        write!(f, "{}", prefix)?;
    }
    if let Some((last, elems)) = v.split_last() {
        for elem in elems {
            write!(f, "{}, ", elem)?;
        }
        write!(f, "{}", last)?;
    }
    write!(f, "\n")?;
    Ok(())
}

impl fmt::Display for ConfigEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:\n", self.name)?;
        format_vec(f, Some("  depends: "), &self.depends)?;
        let enttype = match &self.enttype {
            EntryType::Switch(_) => "switch",
            EntryType::String(_) => "string",
            EntryType::Int(_) => "integer"
        };

        write!(f, "  type: {}\n", enttype)?;
        write!(f, "  choices: ")?;
        match &self.choices {
            Some(choices) => format_vec(f, None, &choices)?,
            None => match &self.enttype {
                EntryType::Switch(_) => write!(f, "y, n\n")?,
                _ => write!(f, "Any {}\n", enttype)?
            }
        };
        let default = match &self.enttype {
            EntryType::Switch(Switch::Yes) => "y".to_string(),
            EntryType::Switch(Switch::No) => "n".to_string(),
            EntryType::String(s) => s.to_string(),
            EntryType::Int(i) => i.to_string()
        };
        write!(f, "  default: {}\n", default)?;
        write!(f, "  help: {}", self.help)?;
        Ok(())
    }
}
