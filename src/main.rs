use crate::cat_file::git_cat_file;
use crate::hash_object::git_hash_object;
use crate::init::git_init;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cat_file;
mod hash_object;
mod init;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        hash_object: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file: PathBuf,
    },
}

#[derive(Clone, Debug)]
enum ObjectKind {
    Blob,
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init => git_init(),
        Command::CatFile {
            pretty_print,
            hash_object,
        } => git_cat_file(pretty_print, hash_object),
        Command::HashObject { write, file } => git_hash_object(file, write),
    }
}
