#![warn(
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]

pub use crate::list::ListOp;
use std::{error, fmt, path};
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
/// Vec wrapper implementing fmt::Display
pub mod display_vec;
/// Logger
pub mod logger;
/// Config generation
pub mod generate;

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
pub enum ConfType {
    Defconfig
}

#[derive(Debug)]
pub enum Mode {
    List {
        ops: Vec<ListOp>
    },
    Validate,
    Enable {
        option: String
    },
    Disable {
        option: String
    },
    Set {
        option: String,
        value: String
    },
    Generate {
        conftype: ConfType
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
    pub depends: display_vec::DisplayVec<String>,
    pub enttype: EntryType,
    pub choices: Option<display_vec::DisplayVec<String>>,
    pub help: String
}

impl ConfigEntry {
    pub fn default_value(&self) -> String {
        match &self.enttype {
            EntryType::Switch(Switch::Yes) => "y".to_string(),
            EntryType::Switch(Switch::No) => "n".to_string(),
            EntryType::String(default) => default.clone(),
            EntryType::Int(int) => int.to_string()
        }
    }

    pub fn is_switch(&self) -> bool {
        match &self.enttype {
            EntryType::Switch(_) => true,
            _ => false
        }
    }

    pub fn is_enabled_by_default(&self) -> Result<bool, Box<dyn error::Error>> {
        match &self.enttype {
            EntryType::Switch(Switch::Yes) => Ok(true),
            EntryType::Switch(Switch::No) => Ok(false),
            _ =>Err(format!("Non-switch option {} cannot be enabled", self.name).into())
        }
    }
}

#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
pub enum Switch {
    Yes,
    No
}

impl fmt::Display for ConfigEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:\n", self.name)?;
        write!(f, "  depends: {}\n", self.depends)?;
        let enttype = match &self.enttype {
            EntryType::Switch(_) => "switch",
            EntryType::String(_) => "string",
            EntryType::Int(_) => "integer"
        };

        write!(f, "  type: {}\n", enttype)?;
        write!(f, "  choices: ")?;
        match &self.choices {
            Some(choices) => write!(f, "{}\n", choices)?,
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
