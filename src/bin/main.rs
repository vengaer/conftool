use conftool::cli;
use conftool::parser;
use std::process::exit;

fn main() {
    let state = cli::args::parse().expect("Invalid arguments, see --help");

    if !state.spec.exists() {
        eprintln!("Specification {} does not exist", state.spec.display());
        exit(1);
    }

    if let Ok(mut cbuf) = state.config.clone().canonicalize() {
        cbuf.pop();
        if cbuf.pop() && !cbuf.exists() {
            eprintln!("Cannot create config in non-existent directory {}", cbuf.display());
            exit(1);
        }
    }

    let entries = parser::parse_spec(&state.spec).unwrap();

    for ent in entries {
        println!("{}", ent.name);
        println!("  depends: {:?}", ent.depends);
        println!("  type: {:?}", ent.enttype);
        println!("  choices: {:?}", ent.choices);
        println!("  help: {}", ent.help);
    }
}
