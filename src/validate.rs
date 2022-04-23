use crate::{display_vec, parse, ConfigEntry, EntryType};
use crate::graph::{state, Graph};
use crate::logger::{Logger, Verbosity};
use regex::Regex;
use std::{collections,hash,error,fmt,fs,path};


fn validate_line_format<T>(lines: &[T], log: &Logger) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Display
{
    let regex = r"^\s*([A-Za-z0-9_-]+\s*=.*)?$";
    let re = Regex::new(regex).unwrap();
    let mut lineno = 0u32;
    let mut valid = true;
    for line in lines {
        lineno += 1;
        log.writeln(Verbosity::Lvl3, &format!("Running {} on line {} against {}", line, lineno, regex));
        if !re.is_match(line.as_ref()) {
            eprintln!("Syntax error on line {}: {}", lineno, line);
            valid = false;
        }
    }

    if !valid {
        return Err("Errors encountered while parsing config file".into());
    }

    Ok(())
}

fn validate_options<T>(kvpairs: &[(T, T)], entries: &[ConfigEntry], log: &Logger) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Display
{
    let entnames: Vec<&str> = entries.iter()
                                     .map(|e| e.name.as_ref())
                                     .collect();
    let mut valid = true;
    for (opt, _) in kvpairs {
        log.writeln(Verbosity::Lvl3, &format!("Checking validity of option {}", opt));
        if !entnames.contains(&opt.as_ref()) {
            eprintln!("Invalid option {}", opt);
            valid = false;
        }
    }

    if !valid {
        return Err("Invalid keys encountered".into());
    }

    Ok(())
}

fn is_integer<T>(s: &T) -> bool
where
    T: AsRef<str>
{
    Regex::new(r"^\d+$").unwrap().is_match(s.as_ref())
}

fn validate_values<T>(kvpairs: &[(T, T)], entries: &[ConfigEntry], log: &Logger) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Display
{
    let mut valid = true;
    for (option, value) in kvpairs {
        log.writeln(Verbosity::Lvl3, &format!("Checking validity of value {} for {}", value, option));
        let ent = entries.iter()
                         .find(|e| e.name == option.as_ref())
                         .unwrap();
        let choices = match ent.enttype {
            EntryType::Switch(_) => vec!["y", "n"],
            _ => {
                if let Some(choices) = &ent.choices {
                    choices.iter()
                           .map(|s| s.as_ref())
                           .collect()
                }
                else {
                    vec![]
                }
            }
        };

        if choices.is_empty() {
            if let EntryType::Int(_) = ent.enttype {
                if !is_integer(value) {
                    eprintln!("Invalid value '{}' for integral option '{}'", value, option);
                    valid = false;
                }
            }
        }
        else if choices.iter().find(|o| *o == &value.as_ref()).is_none() {
            eprintln!("Invalid value '{}' for option '{}'", value, option);
            valid = false;
        }
    }

    if !valid {
        return Err("Encountered unexpectec values".into())
    }

    Ok(())
}

#[derive(Debug)]
enum Cause {
    NotSet,
    NotListed
}

impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cause::NotSet => write!(f, "not set")?,
            Cause::NotListed => write!(f, "not listed")?
        };
        Ok(())
    }
}

fn check_dependencies<T>(graph: &Graph<T, state::Complete>, kvpairs: &[(T, T)], log: &Logger)
    -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Debug + fmt::Display + Clone +
       Eq + hash::Hash
{
    let mut missing: collections::HashMap<T, (Cause, Vec<&T>)> = collections::HashMap::new();
    for (opt, _) in kvpairs {
        log.writeln(Verbosity::Lvl1, &format!("Checking dependencies for {}", opt));
        let deps = graph.dependencies_of(opt)?;
        log.writeln(Verbosity::Lvl3, &format!("Dependencies: {:?}", deps));

        for dep in deps {
            log.writeln(Verbosity::Lvl2, &format!("Checking that {} is set", dep));
            if let Some((_, val)) = kvpairs.iter().find(|(k, _)| k.as_ref() == dep.as_ref()) {
                if val.as_ref() != "y" {
                    if let Some((_, opts)) = missing.get_mut(&dep) {
                        opts.push(opt);
                    }
                    else if !missing.insert(dep.clone(), (Cause::NotSet, vec![opt])).is_none() {
                        return Err(format!("Dependency {} already present in map", dep).into());
                    }
                }
            }
            else {
                if let Some((_, opts)) = missing.get_mut(&dep) {
                    opts.push(opt);
                }
                else if !missing.insert(dep.clone(), (Cause::NotListed, vec![opt])).is_none() {
                    return Err(format!("Dependency {} already present in map", dep).into());
                }
            }
        }
    }

    if !missing.is_empty() {
        for (dep, (cause, opts)) in missing {
            let opts = display_vec::DisplayVec(opts);
            eprintln!("Dependency {} required by {} {}", dep, opts, cause);
        }
        return Err("Errors encountered when evaluating dependencies".into());
    }

    Ok(())
}

pub fn validate_config(path: &path::PathBuf, entries: &[ConfigEntry], log: &Logger)
    -> Result<(), Box<dyn error::Error>>
{
    let lines: Vec<String> = fs::read_to_string(path)?
                                .split("\n")
                                .map(|s| s.to_owned())
                                .collect();
    validate_line_format(&lines, log)?;
    let kvpairs = parse::parse_config(path, Some(lines))?;

    validate_options(&kvpairs, &entries, log)?;
    validate_values(&kvpairs, &entries, log)?;

    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;
    let slice: Vec<(&str, &str)> = kvpairs.iter()
                                          .map(|(k, v)| (k.as_ref(), v.as_ref()))
                                          .collect();
    check_dependencies(&graph, &slice, log)?;
    Ok(())
}
