//!
//! # pp client
//!

#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

pub mod cfg_file;
pub mod cmd_line;
mod ops;

pub use cfg_file::*;
use myutil::{err::*, *};

fn main() {
    pnk!(cfg_file::cfg_init());
    pnk!(cmd_line::parse_and_exec());
}
