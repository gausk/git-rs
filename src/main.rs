use crate::cat_file::git_cat_file;
use crate::commit::git_write_commit;
use crate::hash_object::git_hash_object;
use crate::init::git_init;
use crate::ls_tree::git_ls_tree;
use crate::write_tree::git_write_tree;
use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use std::fs::{read_to_string, write};
use std::path::PathBuf;

mod cat_file;
mod commit;
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
    CommitTree {
        #[clap(short = 'm')]
        message: String,
        #[clap(short = 'p')]
        parent_hash: Option<String>,
        tree_hash: String,
    },
    Commit {
        #[clap(short = 'm')]
        message: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init => {
            git_init()?;
        }
        Command::CatFile {
            pretty_print,
            hash_object,
        } => {
            git_cat_file(pretty_print, hash_object.as_str())?;
        }
        Command::HashObject { write, file } => {
            let hash = git_hash_object(&file, write)?;
            println!("{}", hex::encode(hash));
        }
        Command::LsTree {
            name_only,
            tree_hash,
        } => {
            git_ls_tree(name_only, tree_hash.as_str())?;
        }
        Command::WriteTree => {
            let hash = git_write_tree()?;
            println!("{}", hex::encode(hash));
        }
        Command::CommitTree {
            message,
            parent_hash,
            tree_hash,
        } => {
            let hash = git_write_commit(tree_hash, parent_hash.as_deref(), message)?;
            println!("{}", hex::encode(hash));
        }
        Command::Commit { message } => {
            let tree_hash = git_write_tree()?;
            let head_ref =
                read_to_string(".git/HEAD").with_context(|| "failed to read .git/HEAD")?;
            let head_ref = head_ref.trim();
            let Some(branch_path) = head_ref.strip_prefix("ref: ") else {
                bail!("you can't commit in a headless state");
            };
            let parent_hash = read_to_string(format!(".git/{}", branch_path.trim()))
                .with_context(|| format!("failed to read .git/{}", branch_path.trim()))?;
            let parent_hash = parent_hash.trim();
            ensure!(parent_hash.len() == 40, "bad parent hash");
            let commit_hash = git_write_commit(hex::encode(tree_hash), Some(parent_hash), message)?;
            let commit_hash = hex::encode(commit_hash);
            write(format!(".git/{}", branch_path), &commit_hash)
                .with_context(|| format!("failed to write .git/{}", branch_path))?;
            println!("{commit_hash}");
        }
    }
    Ok(())
}
