use crate::cat_file::git_cat_file;
use crate::hash_object::git_hash_object;
use crate::init::git_init;
use crate::ls_tree::git_ls_tree;
use crate::write_tree::git_write_tree;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cat_file;
mod hash_object;
mod init;
mod ls_tree;
mod object_read;
mod object_write;
mod write_tree;

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
    LsTree {
        #[clap(long)]
        name_only: bool,
        tree_hash: String,
    },
    WriteTree,
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init => git_init(),
        Command::CatFile {
            pretty_print,
            hash_object,
        } => git_cat_file(pretty_print, hash_object.as_str()),
        Command::HashObject { write, file } => {
            let hash = git_hash_object(&file, write)?;
            println!("{}", hex::encode(hash));
            Ok(())
        }
        Command::LsTree {
            name_only,
            tree_hash,
        } => git_ls_tree(name_only, tree_hash.as_str()),
        Command::WriteTree => {
            let hash = git_write_tree()?;
            println!("{}", hex::encode(hash));
            Ok(())
        }
    }
}
