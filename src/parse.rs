use serde;
use serde_json;
use std::error;
use std::fs;
use std::path;
use crate::{display_vec, ConfigEntry, EntryType, Switch};

#[derive(Debug, serde::Deserialize)]
struct ParseEntry {
    /// Name of the entry
    name: String,
    /// Direct dependencies
    depends: Vec<String>,
    /// Entry type and optional default value
    entrytype: String,
    /// Optional set of choices
    choices: Option<Vec<String>>,
    /// Default value
    default: serde_json::Value,
    /// Help string
    help: String
}

#[derive(Debug, serde::Deserialize)]
struct ParseSequence {
    entries: Vec<ParseEntry>
}

pub fn parse_spec(path: &path::PathBuf) -> Result<Vec<ConfigEntry>, Box<dyn error::Error>> {
    let contents = fs::read_to_string(path)?;

    let json: ParseSequence = serde_json::from_str(&contents)?;

    let mut entries: Vec<ConfigEntry> = Vec::with_capacity(json.entries.len());
    for ent in json.entries {
        let defstr = match &ent.default {
            serde_json::Value::String(str) => Some(str),
            _ => None
        };
        let defi32 = match &ent.default {
            serde_json::Value::Number(i) => Some(i.as_i64().unwrap() as i32),
            _ => None
        };
        let enttype = match ent.entrytype.as_str() {
            "integer" => EntryType::Int(defi32.unwrap()),
            "string" => EntryType::String(defstr.unwrap().to_string()),
            "switch" => match defstr.unwrap().as_str() {
                "y" => EntryType::Switch(Switch::Yes),
                "n" => EntryType::Switch(Switch::No),
                _ => return Err(format!("Invalid switch default {}", defstr.unwrap()).into())
            }
            _ => return Err(format!("Invalid entry type {}", ent.entrytype).into())
        };
        let entry = ConfigEntry {
            name: ent.name,
            depends: display_vec::DisplayVec::from(ent.depends),
            enttype: enttype,
            choices: match  ent.choices {
                Some(choices) => Some(display_vec::DisplayVec::from(choices)),
                None => None
            },
            help: ent.help
        };
        entries.push(entry)
    }

    Ok(entries)
}

pub fn parse_config(path: &path::PathBuf, lines: Option<Vec<String>>) -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
    let lines = match lines {
        Some(lines) => lines,
        None => {
            let read: Vec<String> = fs::read_to_string(path)?
                                       .split("\n")
                                       .map(|s| s.to_owned())
                                       .collect();
            read
        }
    };

    let kvpairs = lines.iter()
                       .filter(|&s| !s.is_empty())
                       .map(|s| s.split("=")
                                 .map(|s| s.trim())
                                 .collect())
                       .map(|v: Vec<&str>| (v[0], v[1]))
                       .map(|(k, v)| (k.to_owned(), v.to_owned()))
                       .collect();
    Ok(kvpairs)
}
