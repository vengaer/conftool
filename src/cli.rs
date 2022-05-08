use clap::{Parser, Subcommand};
use crate::{ConfType, ListOp, Mode, State};
use std::error;
use std::path;

const DEFAULT_SPEC: &str = ".conftool.json";
const DEFAULT_CONFIG: &str = ".config";
const MAX_VERBOSITY: usize = 3;

#[derive(Parser, Debug)]
#[clap(
    name = "conftool",
    author = "Vilhelm Engström <vilhelm.engstrom@tuta.io>",
    version = "0.1.3",
    about = "Config file dependency management made easy",
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
    /// Enable config options
    Enable {
        /// Option to enable, automatically handling dependencies
        option: String
    },
    /// Disable config options
    Disable {
        /// Option to disable
        option: String
    },
    /// Set config option
    Set {
        /// The option to set
        option: String,
        /// The value to assign the option
        value: String
    },
    /// Config generation
    Generate {
        /// Type of config to generate
        conftype: String
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
        Some(Subcommands::Enable { option }) => Some(Mode::Enable { option }),
        Some(Subcommands::Disable { option }) => Some(Mode::Disable { option }),
        Some(Subcommands::Set { option, value }) => Some(Mode::Set { option, value }),
        Some(Subcommands::Generate { conftype }) => match conftype.as_ref() {
                "defconfig" => Some(Mode::Generate { conftype: ConfType::Defconfig }),
                _ => None
        },
        None => None
    };
    if mode.is_none() {
        return Err("Invalid syntax".into());
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
