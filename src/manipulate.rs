use std::{error, fs, path};
use std::io::Write;
use crate::{parse, ConfigEntry, EntryType, Switch};
use crate::graph::{state, Graph};

fn enable_deps(opt: &str, kvpairs: &mut Vec<(String, String)>, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;
    let deps = match graph.dependencies_of(&opt) {
        Ok(deps) => deps,
        Err(_) => return Err(format!("Invalid config option \"{}\"", opt).into())
    };

    for dep in &deps {
        for (k, v) in &mut *kvpairs {
            if k == dep {
                *v = "y".to_string();
                break;
            }
        }
    }
    for dep in &deps {
        if kvpairs.iter().find(|(k, _)| k == dep).is_none() {
            kvpairs.push((dep.to_string(), "y".to_string()));
        }
    }

    Ok(())
}

fn write_config(kvpairs: &Vec<(String, String)>, path: &path::PathBuf) -> Result<(), Box<dyn error::Error>> {
    let mut f = fs::File::create(path)?;
    for (k, v) in kvpairs {
        f.write_all(format!("{} = {}\n", k, v).as_ref())?;
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

pub fn enable(opt: &str, path: &path::PathBuf, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let mut kvpairs = parse::parse_config(path, None)?;
    enable_deps(opt, &mut kvpairs, &entries)?;
    set_switch(opt, Switch::Yes, &mut kvpairs, &entries)?;
    write_config(&kvpairs, path)
}

pub fn disable(opt: &str, path: &path::PathBuf, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let mut kvpairs = parse::parse_config(path, None)?;
    set_switch(opt, Switch::No, &mut kvpairs, &entries)?;
    write_config(&kvpairs, path)
}
