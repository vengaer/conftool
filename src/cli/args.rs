use clap::{Parser, Subcommand};
use crate::{State, Mode, ListOp};
use std::error;
use std::path::PathBuf;

const DEFAULT_SPEC: &str = ".conftool.json";
const DEFAULT_CONFIG: &str = ".config";
const MAX_VERBOSITY: usize = 3;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
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
        all: bool
    }
}

pub fn parse() -> Result<State, Box<dyn error::Error>> {
    let args = CliArgs::parse();

    let verbosity = match args.verbose {
        0..=MAX_VERBOSITY => args.verbose,
        _ =>  {
            eprintln!("Warning: Verbosity clamped at {}", MAX_VERBOSITY);
            MAX_VERBOSITY
        }
    };

    let mode = match args.subcmd {
        Some(Subcommands::List { show, all }) => {
            if all {
                Some(Mode::List { op: ListOp::All })
            }
            else if let Some(show) = show {
                Some(Mode::List { op: ListOp::Show(show) })
            }
            else {
                None
            }
        },
        None => None
    };

    if let None = mode {
        return Err("No submode specified".into());
    }

    Ok(State {
        spec: match args.specification {
            Some(spec) => PathBuf::from(spec),
            None => PathBuf::from(DEFAULT_SPEC)
        },
        config : match args.config {
            Some(cfg) => PathBuf::from(cfg),
            None => PathBuf::from(DEFAULT_CONFIG)
        },
        verbosity,
        mode: mode.unwrap()
    })
}
