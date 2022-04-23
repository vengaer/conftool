use std::{error, path};
use crate::logger::{Logger, Verbosity};
use crate::{manipulate, ConfigEntry, EntryType, Switch};

pub fn defconfig(path: &path::PathBuf, entries: &[ConfigEntry], log: &Logger) -> Result<(), Box<dyn error::Error>> {
    let mut kvpairs: Vec<(&str, String)> = Vec::with_capacity(entries.len());
    for ent in entries {
        let default = match &ent.enttype {
            EntryType::Switch(Switch::Yes) => "y".to_string(),
            EntryType::Switch(Switch::No) => "n".to_string(),
            EntryType::String(default) => default.clone(),
            EntryType::Int(i) => i.to_string()
        };

        log.writeln(Verbosity::Lvl2, &format!("Choosing default \"{}\" for option \"{}\"", default, ent.name));
        kvpairs.push((&ent.name, default));
    }

    manipulate::write_config(&kvpairs, path, log)
}
