use std::{error, path};
use crate::logger::{Logger, Verbosity};
use crate::graph::{state, Graph};
use crate::{manipulate, ConfigEntry};

pub fn defconfig(path: &path::PathBuf, entries: &[ConfigEntry], log: &Logger) -> Result<(), Box<dyn error::Error>> {
    let mut kvpairs: Vec<(&str, String)> = Vec::with_capacity(entries.len());
    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;

    let mut enable;
    for ent in entries {
        let deps = graph.dependencies_of(&ent.name.as_ref())?;
        log.writeln(Verbosity::Lvl1, &format!("Checking dependencies of \"{}\"", ent.name));

        enable = true;
        for dep in deps {
            let dep = entries.iter()
                             .find(|e| e.name == dep)
                             .unwrap();
            if !dep.is_switch() {
                return Err(format!("Option {} depends on non-switch option {} which is not supported", ent.name, dep).into());
            }

            // Safe to unwrap due to the above check
            if !dep.is_enabled_by_default().unwrap() {
                log.writeln(Verbosity::Lvl1, &format!("Skipping \"{}\" due to disabled dependency \"{}\"", ent, dep.name));
                enable = false;
                break;
            }
        }

        if enable {
            let default = ent.default_value();
            log.writeln(Verbosity::Lvl2, &format!("Choosing default \"{}\" for option \"{}\"", default, ent.name));
            kvpairs.push((&ent.name, default));
        }
    }

    manipulate::write_config(&kvpairs, path, log)
}
