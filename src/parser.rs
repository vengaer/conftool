use serde::{Deserialize};
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub enum EntryType {
    Switch(Switch),
    String(String),
    Int(i32)
}

#[derive(Debug, Deserialize)]
struct ParseEntry {
    /// Name of the entry
    name: String,
    /// Direct dependencies
    depends: Vec<String>,
    /// Entry type and optional default value
    entrytype: String,
    /// Optional set of chioces
    choices: Option<Vec<String>>,
    /// Default value
    default: Value,
    /// Help string
    help: String
}

#[derive(Debug, Deserialize)]
struct ParseSequence {
    entries: Vec<ParseEntry>
}

#[derive(Debug, Deserialize)]
pub enum Switch {
    Yes,
    No
}

#[derive(Debug)]
pub struct ConfigEntry {
    pub name: String,
    pub depends: Vec<String>,
    pub enttype: EntryType,
    pub choices: Option<Vec<EntryType>>,
    pub help: String
}

pub fn parse_spec(path: &PathBuf) -> Result<Vec<ConfigEntry>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;

    let json: ParseSequence = serde_json::from_str(&contents)?;

    let mut entries: Vec<ConfigEntry> = Vec::with_capacity(json.entries.len());
    for ent in json.entries {
        let defstr = match &ent.default {
            Value::String(str) => Some(str),
            _ => None
        };
        let defi32 = match &ent.default {
            Value::Number(i) => Some(i.as_i64().unwrap() as i32),
            _ => None
        };
        let enttype = match ent.entrytype.as_str() {
            "integer" => EntryType::Int(defi32.unwrap()),
            "string" => EntryType::String(defstr.unwrap().to_string()),
            "switch" => match defstr.unwrap().as_str() {
                "y" => EntryType::Switch(Switch::Yes),
                "n" => EntryType::Switch(Switch::No),
                _ => { panic!("Invalid switch default {}", defstr.unwrap()); }
            }
            _ => { panic!("Invalid entry type {}", ent.entrytype); }
        };
        let entry = ConfigEntry {

            name: ent.name,
            depends: ent.depends,
            enttype: enttype,
            choices: None,
            help: ent.help
        };
        entries.push(entry)
    }

    Ok(entries)
}
