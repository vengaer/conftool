use conftool::{cli,parser,list,Mode,ListOp};
use std::process::exit;

fn main() {
    let state = cli::args::parse().expect("Invalid arguments, try --help");

    if !state.spec.exists() {
        eprintln!("Specification {} does not exist", state.spec.display());
        exit(1);
    }

    if let Ok(mut cbuf) = state.config.clone().canonicalize() {
        cbuf.pop();
        if !cbuf.exists() {
            eprintln!("Cannot create config in non-existent directory {}", cbuf.display());
            exit(1);
        }
    }

    let entries = parser::parse_spec(&state.spec).unwrap();

    match state.mode {
        Mode::List { op } => {
            match op {
                ListOp::Show(option) => {
                    list::show(&option, &entries).unwrap()
                },
                ListOp::All => {
                    list::show_all(&entries)
                }
            }
        }
    }
}
