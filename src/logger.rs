use std::fmt;

#[derive(Debug)]
pub enum Verbosity {
    Lvl0,
    Lvl1,
    Lvl2,
    Lvl3
}

impl Verbosity {
    pub fn new(lvl: usize) -> Self {
        match lvl {
            0 => Verbosity::Lvl0,
            1 => Verbosity::Lvl1,
            2 => Verbosity::Lvl2,
            _ => Verbosity::Lvl3
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            Verbosity::Lvl0 => 0usize,
            Verbosity::Lvl1 => 1usize,
            Verbosity::Lvl2 => 2usize,
            Verbosity::Lvl3 => 3usize
        }
    }
}

#[derive(Debug)]
pub struct Logger {

    /// Verbosity level
    verbosity: Verbosity
}

impl Logger {
    pub fn new(verbosity: usize) -> Self {
        Logger { verbosity: Verbosity::new(verbosity) }
    }

    pub fn writeln<T>(&self, lvl: Verbosity, s: &T)
        where T: fmt::Display
    {
        if lvl.as_usize() > self.verbosity.as_usize() {
            return
        }
        println!("{}", s);
    }
}
