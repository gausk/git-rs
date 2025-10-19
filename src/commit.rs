use crate::object_read::{Object, ObjectKind};
use anyhow::{Context, Result};
use chrono::Local;
use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

/// A *commit object* in Git represents a snapshot of the repository at a point in time,
/// along with metadata about the author, committer, and commit message.
///
/// A commit object references a single tree object (the root directory snapshot)
/// and optionally one or more parent commits (for merges).
///
/// The raw (uncompressed) format of a commit object is a series of ASCII lines:
///
///     tree <tree-id>
///     parent <parent-id>  # optional, repeatable for multiple parents
///     author <name> <email> <timestamp> <timezone>
///     committer <name> <email> <timestamp> <timezone>
///
///     <commit message>
///
/// - `<tree-id>`: 40-character hex SHA-1 (or SHA-256) of the root tree object
/// - `<parent-id>`: 40-character hex SHA-1 (or SHA-256) of a parent commit
/// - `<name>`: author/committer name
/// - `<email>`: author/committer email
/// - `<timestamp>`: seconds since Unix epoch
/// - `<timezone>`: numeric timezone offset (e.g., `+0530`)
/// - `<commit message>`: arbitrary UTF-8 text describing the changes
///
/// The full uncompressed content begins with the header:
///     "commit <size>\0<content>"
///
/// Steps:
/// 1. Construct the commit content lines as shown above.
/// 2. Prefix it with `"commit <size>\0"`, where `<size>` is the byte length of the content.
/// 3. Compute the SHA-1 (or SHA-256) hash of the uncompressed data.
/// 4. Hex-encode the hash to get the commit’s object ID.
/// 5. Compress the data using zlib.
/// 6. Store it under:
///    .git/objects/<first 2 hex chars>/<remaining 38 chars>
///
/// Example:
///
///   Commit referencing tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904:
///
///     tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904
///     author Alice <alice@example.com> 1697750400 +0530
///     committer Alice <alice@example.com> 1697750400 +0530
///
///     Initial commit
///
///   SHA-1 hash:
///     e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
///   Stored at:
///     .git/objects/e6/9de29bb2d1d6434b8b29ae775ad8c2e48c5391
///
/// Note: Commits form a chain — each commit references its parent(s), allowing
///       Git to track history and perform merges.
///
pub fn git_write_commit(
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
) -> Result<[u8; 20]> {
    let mut out = String::new();
    writeln!(out, "tree {}", tree_hash)?;
    if let Some(parent_hash) = parent_hash {
        writeln!(out, "parent {}", parent_hash)?;
    }
    let (time, tz) = get_time_and_timezone();
    let (name, email) = get_name_and_email_from_git_config()?;
    writeln!(out, "author {} <{}> {} {}", name, email, time, tz)?;
    writeln!(out, "committer {} <{}> {} {}", name, email, time, tz)?;
    writeln!(out)?;
    writeln!(out, "{}", message)?;
    let mut object = Object {
        kind: ObjectKind::Commit,
        expected_size: out.len() as u64,
        reader: Cursor::new(out),
    };
    let hash = object.write_as_object()?;
    Ok(hash)
}

fn get_time_and_timezone() -> (i64, String) {
    let now = Local::now();
    let time = now.timestamp();
    let offset_seconds = now.offset().local_minus_utc();
    let hours = offset_seconds / 3600;
    let minutes = offset_seconds.abs() % 3600 / 60;
    let tz = format!("{:+03}{:02}", hours, minutes);
    (time, tz)
}

fn get_name_and_email_from_git_config() -> Result<(String, String)> {
    let mut path = env::home_dir().context("Couldn't determine home directory")?;
    path.push(".gitconfig");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open git config file at {:?}", path))?;
    let reader = BufReader::new(file);
    let mut name = String::new();
    let mut email = String::new();
    for line in reader.lines() {
        let line = line.context("Failed to read git config file line")?;
        if let Some(nm) = line.strip_prefix("name = ") {
            name = nm.trim().to_string();
        } else if let Some(em) = line.strip_prefix("email = ") {
            email = em.trim().to_string();
        }
        if !name.is_empty() && !email.is_empty() {
            break;
        }
    }
    Ok((name, email))
}
