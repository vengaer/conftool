use conftool::cli;
use std::process::exit;

fn main() {
    let state = cli::args::parse().expect("Invalid arguments, see --help");

    if !state.spec.exists() {
        eprintln!("Specification {} does not exist", state.spec.display());
        exit(1);
    }

    let mut cbuf = state.config.clone();
    cbuf.pop();
    if !cbuf.exists() {
        eprintln!("Cannot create config in non-existent directory {}", cbuf.display());
        exit(1);
    }

}
