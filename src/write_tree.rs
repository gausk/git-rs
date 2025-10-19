use crate::hash_object::git_hash_object;
use crate::object_read::{Object, ObjectKind};
use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use std::cmp::Ordering;
use std::fs::Metadata;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// A *tree object* in Git represents a directory snapshot.
///
/// Each tree entry maps a filename to a blob (file) or another tree (subdirectory),
/// along with its file mode (permissions).
///
/// The raw (uncompressed) format of a tree object is a concatenation of entries:
///
///     "<file mode> <file name>\0<20-byte binary object id>"
///
/// - `<file mode>`: ASCII digits like `100644` (normal file), `100755` (executable), or `40000` (directory)
/// - `<file name>`: the file or directory name (no path separators)
/// - `<20-byte binary object id>`: raw SHA-1/SHA-256 bytes of the referenced blob or tree
///
/// The full uncompressed content begins with the header:
///     "tree <size>\0<entries>"
///
/// Steps:
/// 1. Construct the concatenated entry list from directory contents.
/// 2. Prefix it with `"tree <size>\0"`, where `<size>` is the total byte length of entries.
/// 3. Compute the SHA-1 (or SHA-256) hash of this uncompressed data.
/// 4. Hex-encode the hash to get the tree’s object ID.
/// 5. Compress the data using zlib.
/// 6. Store it under:
///    .git/objects/<first 2 hex chars>/<remaining 38 chars>
///
/// Example (a directory containing a single file `hello.txt`):
///
///   Entry:
///     100644 hello.txt\0<20-byte blob-id>
///   Uncompressed form:
///     tree 37\0<entry bytes>
///   SHA-1 hash:
///     4b825dc642cb6eb9a060e54bf8d69288fbee4904
///   Stored at:
///     .git/objects/4b/825dc642cb6eb9a060e54bf8d69288fbee4904
///
/// Note: Tree objects form a hierarchy — a commit object references
///       the root tree, which may reference subtrees and blobs recursively.
///
pub fn git_write_tree() -> Result<[u8; 20]> {
    let Some(hash) = git_write_tree_with_path(Path::new("."))? else {
        bail!("empty git repo")
    };
    Ok(hash)
}
pub fn git_write_tree_with_path(path: &Path) -> Result<Option<[u8; 20]>> {
    let walker = WalkBuilder::new(path)
        .max_depth(Some(1))
        .hidden(false)
        .build();
    // sadly ignore::Walk does not provide an easy way to ignore itself or .git
    let mut entries: Vec<_> = walker
        .filter_map(|e| {
            let entry = e.ok()?;
            if entry.depth() == 0 || entry.file_name() == ".git" {
                None
            } else {
                Some(entry)
            }
        })
        .collect();
    // In Git for directories we add / at the end for sorting
    entries.sort_unstable_by(|a, b| {
        let af = a.file_name().as_encoded_bytes();
        let bf = b.file_name().as_encoded_bytes();
        let min_len = af.len().min(bf.len());
        match af[..min_len].cmp(&bf[..min_len]) {
            Ordering::Equal => {}
            other => return other,
        }
        let a1 = af
            .get(min_len)
            .copied()
            .or(a.path().is_dir().then_some(b'/'));
        let b1 = bf
            .get(min_len)
            .copied()
            .or(b.path().is_dir().then_some(b'/'));
        a1.cmp(&b1)
    });
    let mut out = Vec::new();
    for entry in entries {
        let path = entry.path();
        let hash = if path.is_dir() {
            let Some(hash) = git_write_tree_with_path(path)? else {
                continue;
            };
            hash
        } else {
            git_hash_object(path, true)?
        };
        let mode = get_mode_for_entry(&entry.metadata().context("reading metadata")?);
        out.extend_from_slice(mode.as_bytes());
        out.push(b' ');
        out.extend_from_slice(entry.file_name().as_encoded_bytes());
        out.push(0);
        out.extend(hash);
    }
    if out.is_empty() {
        Ok(None)
    } else {
        let mut object = Object {
            kind: ObjectKind::Tree,
            expected_size: out.len() as u64,
            reader: Cursor::new(out),
        };
        let hash = object.write_as_object()?;
        Ok(Some(hash))
    }
}

pub fn get_mode_for_entry(meta: &Metadata) -> &'static str {
    if meta.is_dir() {
        "40000"
    } else if meta.is_symlink() {
        "120000"
    } else if meta.permissions().mode() & 0o111 != 0 {
        "100755"
    } else {
        "100644"
    }
}
