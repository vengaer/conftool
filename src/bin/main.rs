use conftool::{cli,parser,list,validate,Mode,ListOp};
use std::process;

fn main() {
    let state = match cli::parse_args() {
        Ok(state) => state,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

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

    let entries = parser::parse_spec(&state.spec).unwrap();
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
        Mode::Validate => {
            validate::validate_config(&state.config, &entries)
        }
    };

    if let Err(err) = res {
        eprintln!("{}", err);
        process::exit(1);
    }
}
