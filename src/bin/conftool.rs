use std::process;
use conftool::{cli, generate, list, logger, parse, manipulate, validate};
use conftool::{ConfType, ListOp, Mode};

fn main() {
    let state = match cli::parse_args() {
        Ok(state) => state,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let log = logger::Logger::new(state.verbosity);

    if !state.spec.exists() {
        eprintln!("Specification {} does not exist", state.spec.display());
        process::exit(1);
    }

    if let Ok(mut cbuf) = state.config.clone().canonicalize() {
        cbuf.pop();
        if !cbuf.exists() {
            eprintln!("Cannot create config in non-existent directory {}", cbuf.display());
            process::exit(1);
        }
    }

    let entries = parse::parse_spec(&state.spec).unwrap();
    let res = match state.mode {
        Mode::List { mut ops } => {
            loop {
                let op = match ops.pop() {
                    Some(op) => op,
                    None => break Ok(())
                };
                let res = match op {
                    ListOp::Show(option) => list::show(&option, &entries),
                    ListOp::All => {
                        list::show_all(&entries);
                        Ok(())
                    },
                    ListOp::Dependencies(option) => list::dependencies(&option, &entries)
                };
                if res.is_err() {
                    break res;
                }
            }
        },
        Mode::Validate => validate::validate_config(&state.config, &entries, &log),
        Mode::Enable { option } => manipulate::enable(&option, &state.config, &entries, &log),
        Mode::Disable { option } => manipulate::disable(&option, &state.config, &entries, &log),
        Mode::Set { option, value } => manipulate::set(&option, &value, &state.config, &entries, &log),
        Mode::Generate { conftype } => match conftype {
            ConfType::Defconfig => generate::defconfig(&state.config, &entries, &log)
        }
    };

    if let Err(err) = res {
        eprintln!("{}", err);
        process::exit(1);
    }
}
