use regex::Regex;
use std::{error, fs, path};
use std::io::Write;
use crate::{parse, ConfigEntry, EntryType, Switch};
use crate::graph::{state, Graph};
use crate::logger::{Logger, Verbosity};

fn enable_dependencies(opt: &str, kvpairs: &mut Vec<(String, String)>, entries: &[ConfigEntry], log: &Logger)
    -> Result<(), Box<dyn error::Error>>
{
    log.writeln(Verbosity::Lvl1, &format!("Enabling dependencies for {}", opt));
    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;
    let deps = match graph.dependencies_of(&opt) {
        Ok(deps) => deps,
        Err(_) => return Err(format!("Invalid config option \"{}\"", opt).into())
    };

    log.writeln(Verbosity::Lvl2, &format!("Dependencies for {}: {:?}", opt, deps));

    for dep in &deps {
        if let Some((_, v)) = kvpairs.iter_mut().find(|(k, _)| k == dep) {
            log.writeln(Verbosity::Lvl2, &format!("Enabling existing dependency \"{}\"", dep));
            *v = "y".to_string();
        }
        else {
            log.writeln(Verbosity::Lvl2, &format!("Adding missing depdency \"{}\"", dep));
            kvpairs.push((dep.to_string(), "y".to_string()));
        }
    }

    Ok(())
}

fn disable_dependent(opt: &str, kvpairs: &mut Vec<(String, String)>, entries: &[ConfigEntry], log: &Logger)
    -> Result<(), Box<dyn error::Error>>
{
    log.writeln(Verbosity::Lvl1, &format!("Disabling options depending on {}", opt));
    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;
    let dependent = match graph.dependent_vertices(&opt) {
        Ok(dependent) => dependent,
        Err(_) => return Err(format!("Invalid config option \"{}\"", opt).into())
    };

    log.writeln(Verbosity::Lvl2, &format!("Options depending on {}: {:?}", opt, dependent));

    for dep in &dependent {
        let ent = entries.iter().find(|e| e.name == *dep).unwrap();
        match ent.enttype {
            EntryType::Switch(_) => {
                if let Some((_, v)) = kvpairs.iter_mut().find(|(k, _)| k == dep) {
                    log.writeln(Verbosity::Lvl2, &format!("Disabling exising option \"{}\"", dep));
                    *v = "n".to_string();
                }
                else {
                    log.writeln(Verbosity::Lvl2, &format!("Disabling missing option \"{}\"", dep));
                    kvpairs.push((dep.to_string(), "n".to_string()));
                }
            },
            _ => {
                log.writeln(Verbosity::Lvl2, &format!("Removing dependent option \"{}\"", dep));
                kvpairs.retain(|(k, _)| k != dep);
            }
        }
    }

    Ok(())
}

fn write_config(kvpairs: &Vec<(String, String)>, path: &path::PathBuf, log: &Logger)
    -> Result<(), Box<dyn error::Error>>
{
    let mut f = fs::File::create(path)?;
    log.writeln(Verbosity::Lvl2, &format!("Writing config to {}", path.to_str().unwrap()));
    let pad = kvpairs.iter()
                     .map(|(k, _)| k.len())
                     .max()
                     .unwrap();
    log.writeln(Verbosity::Lvl3, &format!("Padding keys to {} chars", pad));
    for (k, v) in kvpairs {
        f.write_all(format!("{:width$} = {}\n", k, v, width=pad).as_ref())?;
    }
    Ok(())
}

fn set_switch(opt: &str, desired: Switch, kvpairs: &mut Vec<(String, String)>, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let ent = match entries.iter().find(|e| e.name == opt) {
        Some(ent) => ent,
        None => return Err(format!("Invalid config option \"{}\"", opt).into())
    };

    match ent.enttype {
        EntryType::Switch(_) => (),
        _ => {
            let action = match desired {
                Switch::Yes => "enable",
                Switch::No => "disable"
            };
            return Err(format!("Cannot {} non-switch option \"{}\"", action, opt).into())
        }
    };

    let desired = match desired {
        Switch::Yes => "y".to_string(),
        Switch::No => "n".to_string()
    };

    if let Some((_, v)) = kvpairs.iter_mut().find(|(k, _)| k == opt) {
        *v = desired;
    }
    else {
        kvpairs.push((opt.to_string(), desired));
    }
    Ok(())
}

pub fn enable(opt: &str, path: &path::PathBuf, entries: &[ConfigEntry], log: &Logger)
    -> Result<(), Box<dyn error::Error>>
{
    let mut kvpairs = parse::parse_config(path, None)?;
    enable_dependencies(opt, &mut kvpairs, &entries, log)?;
    log.writeln(Verbosity::Lvl1, &format!("Enabling switch {}", opt));
    set_switch(opt, Switch::Yes, &mut kvpairs, &entries)?;
    write_config(&kvpairs, path, log)
}

pub fn disable(opt: &str, path: &path::PathBuf, entries: &[ConfigEntry], log: &Logger)
    -> Result<(), Box<dyn error::Error>>
{
    let mut kvpairs = parse::parse_config(path, None)?;
    disable_dependent(opt, &mut kvpairs, &entries, log)?;
    log.writeln(Verbosity::Lvl1, &format!("Disabling switch {}", opt));
    set_switch(opt, Switch::No, &mut kvpairs, &entries)?;
    write_config(&kvpairs, path, log)
}

fn is_integer(s: &str) -> bool {
    return Regex::new(r"^\s*[0-9]+\s*$").unwrap()
                                        .is_match(s);
}

fn validate_value(opt: &str, value: &str, ent: &ConfigEntry) -> Result<(), Box<dyn error::Error>> {
    match ent.enttype {
        EntryType::Switch(_) => {
            if value != "y" && value != "n" {
                return Err(format!("Invalid value \"{}\" for switch \"{}\"", value, opt).into());
            }
        },
        EntryType::String(_) => (),
        EntryType::Int(_) => {
            if !is_integer(&value) {
                return Err(format!("Invalid value \"{}\" for integer \"{}\"", value, opt).into());
            }
        }
    };

    if let Some(choices) = &ent.choices {
        if choices.iter().find(|v| v == &value).is_none() {
            return Err(format!("Invalid choice \"{}\" for option \"{}\"\nValid options are {:?}",
                               value, opt, choices).into());
        }
    }
    Ok(())
}

pub fn set(opt: &str, value: &str, path: &path::PathBuf, entries: &[ConfigEntry], log: &Logger)
    -> Result<(), Box<dyn error::Error>>
{
    log.writeln(Verbosity::Lvl3, &format!("Looking up find option {}...", opt));
    let ent = match entries.iter().find(|e| e.name == opt) {
        Some(ent) => ent,
        None => return Err(format!("Invalid config option \"{}\"", opt).into())
    };
    log.writeln(Verbosity::Lvl3, &format!("Option \"{}\" is known", opt));

    let value = value.trim();

    validate_value(opt, value, &ent)?;
    log.writeln(Verbosity::Lvl3, &format!("Value \"{}\" is valid for option \"{}\"", value, opt));

    let mut kvpairs = parse::parse_config(path, None)?;
    match ent.enttype {
        EntryType::Switch(_) => {
            if value == "n" {
                disable_dependent(opt, &mut kvpairs, &entries, log)?;
            }
            else {
                enable_dependencies(opt, &mut kvpairs, &entries, log)?;
            }
        },
        _ => enable_dependencies(opt, &mut kvpairs, &entries, log)?
    };

    if let Some((_, v)) = kvpairs.iter_mut().find(|(k, _)| k == opt) {
        log.writeln(Verbosity::Lvl1, &format!("Setting value \"{}\" for existing option \"{}\"", value, opt));
        *v = value.to_string();
    }
    else {
        log.writeln(Verbosity::Lvl1, &format!("Setting value \"{}\" for missing option \"{}\"", value, opt));
        kvpairs.push((opt.to_string(), value.to_string()));
    }

    write_config(&kvpairs, path, log)
}
