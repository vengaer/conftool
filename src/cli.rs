use clap::{Parser, Subcommand};
use crate::{State, Mode, ListOp};
use std::error;
use std::path;

const DEFAULT_SPEC: &str = ".conftool.json";
const DEFAULT_CONFIG: &str = ".config";
const MAX_VERBOSITY: usize = 3;

#[derive(Parser, Debug)]
#[clap(
    author = "Vilhelm Engström",
    version = "0.1.0",
    about = "Simple dependency handling in config files",
    long_about = None
)]
struct CliArgs {
    /// Path to config specification, optional
    #[clap(short, long, value_name = "SPECIFICATION")]
    specification: Option<String>,

    /// Path of config file, optional
    #[clap(short, long, value_name = "CONFIG")]
    config: Option<String>,

    /// Increase verbosity, may be passed repeatedly
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    #[clap(subcommand)]
    subcmd: Option<Subcommands>
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    /// List configuration options
    List {
        /// Show information regarding configuration option
        #[clap(short, long, value_name = "OPTION")]
        show: Option<String>,

        /// Show information about all options
        #[clap(short, long)]
        all: bool,

        /// List dependencies of option, including indirect ones
        #[clap(short, long = "dependencies", value_name = "OPTION")]
        deps: Option<String>
    },
    /// Validate config file
    Validate,
    /// Set config options
    Set {
        /// Set a specific option, automatically enables dependencies
        #[clap(short, long, value_name = "OPTION")]
        option: Option<String>
    }
}

pub fn parse_args() -> Result<State, Box<dyn error::Error>> {
    let args = CliArgs::parse();

    let verbosity = match args.verbose {
        0..=MAX_VERBOSITY => args.verbose,
        _ =>  {
            eprintln!("Warning: Verbosity clamped at {}", MAX_VERBOSITY);
            MAX_VERBOSITY
        }
    };

    let mode = match args.subcmd {
        Some(Subcommands::List { show, all, deps }) => {
            if all {
                Some(Mode::List { ops: vec![ListOp::All] })
            }
            else {
                match (&show, &deps) {
                    (None, None) => None,
                    _ => {
                        let mut ops = vec![];
                        if !show.is_none() {
                            ops.push(ListOp::Show(show.unwrap()));
                        }
                        if !deps.is_none() {
                            ops.push(ListOp::Dependencies(deps.unwrap()));
                        }
                        Some(Mode::List { ops })
                    }
                }
            }
        }
        Some(Subcommands::Validate) => Some(Mode::Validate),
        Some(Subcommands::Set { option }) => {
            match option {
                Some(option) => Some(Mode::Set { option }),
                _ => None
            }
        },
        None => None
    };
    if mode.is_none() {
        return Err("No submode specified".into());
    }

    Ok(State {
        spec: match args.specification {
            Some(spec) => path::PathBuf::from(spec),
            None => path::PathBuf::from(DEFAULT_SPEC)
        },
        config : match args.config {
            Some(cfg) => path::PathBuf::from(cfg),
            None => path::PathBuf::from(DEFAULT_CONFIG)
        },
        verbosity,
        mode: mode.unwrap()
    })
}
