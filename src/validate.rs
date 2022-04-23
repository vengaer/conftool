use crate::{ConfigEntry,EntryType};
use crate::graph::{state, Graph};
use regex::Regex;
use std::{collections,hash,error,fmt,fs,ops,path};


fn validate_line_format<T>(lines: &[T]) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Display
{
    let re = Regex::new(r"^\s*([A-Za-z0-9_-]+\s*=.*)?$").unwrap();
    let mut lineno = 0u32;
    let mut valid = true;
    for line in lines {
        lineno += 1;
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

fn validate_options<T>(kvpairs: &[(T, T)], entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Display
{
    let entnames: Vec<&str> = entries.iter()
                                     .map(|e| e.name.as_ref())
                                     .collect();
    let mut valid = true;
    for (opt, _) in kvpairs {
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

fn validate_values<T>(kvpairs: &[(T, T)], entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Display
{
    let mut valid = true;
    for (option, value) in kvpairs {
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

        if choices.len() == 0usize {
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

#[derive(Debug)]
struct DisplayVec<T>(Vec<T>);

impl<T> ops::Deref for DisplayVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> fmt::Display for DisplayVec<T>
where
    T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((last, rest)) = self.split_last() {
            for elem in rest {
                write!(f, "{}, ", elem)?;
            }
            write!(f, "{}", last)?;
        }
        Ok(())
    }
}

impl<T> From<Vec<T>> for DisplayVec<T> {
    fn from(v: Vec<T>) -> Self {
        DisplayVec(v)
    }
}

fn check_dependencies<'a, T>(graph: &Graph<T, state::Complete>, kvpairs: &'a [(T, T)]) -> Result<(), Box<dyn error::Error>>
where
    T: AsRef<str> + fmt::Debug + fmt::Display + Clone +
       Eq + PartialEq<&'a str> + hash::Hash
{
    let mut missing: collections::HashMap<T, (Cause, Vec<&T>)> = collections::HashMap::new();
    for (opt, _) in kvpairs {
        let deps = graph.dependencies_of(opt)?;

        for dep in deps {
            if let Some((_, val)) = kvpairs.iter().find(|(k, _)| k == &dep) {
                if val != &"y" {
                    if let Some((_, opts)) = missing.get_mut(&dep) {
                        opts.push(opt);
                    }
                    else if !missing.insert(dep.clone(), (Cause::NotSet, vec![])).is_none() {
                        return Err(format!("Dependency {} already present in map", dep).into());
                    }
                }
            }
            else {
                if let Some((_, opts)) = missing.get_mut(&dep) {
                    opts.push(opt);
                }
                else if !missing.insert(dep.clone(), (Cause::NotListed, vec![])).is_none() {
                    return Err(format!("Dependency {} already present in map", dep).into());
                }
            }
        }
    }

    if missing.len() > 0usize {
        for (dep, (cause, opts)) in missing {
            let opts = DisplayVec(opts);
            eprintln!("Dependency {} required by {} {}", dep, opts, cause);
        }
        return Err("Errors encountered when evaluating dependencies".into());
    }

    Ok(())
}

pub fn validate_config(path: &path::PathBuf, entries: &[ConfigEntry]) -> Result<(), Box<dyn error::Error>> {
    let lines: Vec<String> = fs::read_to_string(path)?
                                .split("\n")
                                .map(|s| s.to_owned())
                                .collect();

    let kvpairs: Vec<(&str, &str)> = lines.iter()
                                          .filter(|&s| !s.is_empty())
                                          .map(|s| s.split("=")
                                                    .map(|s| s.trim())
                                                    .collect())
                                          .map(|v: Vec<&str>| (v[0], v[1]))
                                          .collect();

    validate_line_format(&lines)?;
    validate_options(&kvpairs, &entries)?;
    validate_values(&kvpairs, &entries)?;

    let graph = Graph::<&str, state::Incomplete>::from(entries);
    let graph = graph.into_complete()?;
    check_dependencies(&graph, &kvpairs)?;
    Ok(())
}
