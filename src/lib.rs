#![warn(
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]

use std::path::PathBuf;

/// Command line management
pub mod cli;

#[derive(Debug)]
pub struct State {
    /// Path to config specification
    pub spec: PathBuf,

    /// Path to config file
    pub config: PathBuf,

    /// Verbosity level
    pub verbosity: usize,

    /// Mode of operation
    pub mode: Mode
}

#[derive(Debug)]
pub enum Mode {
    List {
        show: String
    }
}
