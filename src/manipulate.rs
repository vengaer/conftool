use std::{error, fs, path};
use std::io::Write;
use crate::{parse, ConfigEntry};
use crate::graph::{state, Graph};

fn enable_deps(opt: &str, kvpairs: &mut Vec<(String, String)>, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;
    let deps = graph.dependencies_of(&opt)?;

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

pub fn enable(opt: &str, path: &path::PathBuf, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let mut kvpairs = parse::parse_config(path, None)?;
    enable_deps(opt, &mut kvpairs, &entries)?;
    write_config(&kvpairs, path)?;
    Ok(())
}
