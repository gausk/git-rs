use crate::object_read::{Object, ObjectKind, read_git_object};
use anyhow::{Context, Result, anyhow, bail};
use std::ffi::CStr;
use std::io::{BufRead, Write, stdout};

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
pub fn git_ls_tree(name_only: bool, tree_hash: &str) -> Result<()> {
    let object = read_git_object(tree_hash)?;
    match object.kind {
        ObjectKind::Tree => git_read_tree_content(object, name_only),
        _ => Err(anyhow!("not a tree object")),
    }
}

pub fn git_read_tree_content<R: BufRead>(mut object: Object<R>, name_only: bool) -> Result<()> {
    let mut buf = Vec::new();
    let mut sout = stdout().lock();
    let mut hash_buf = [0; 20];
    // TODO: read only expected size
    loop {
        buf.clear();
        let n = object
            .reader
            .read_until(0, &mut buf)
            .context("invalid tree entry")?;
        if n == 0 {
            break;
        }
        let mode_and_name =
            CStr::from_bytes_with_nul(buf.as_slice()).context("invalid tree entry format")?;
        let mode_and_name = mode_and_name
            .to_str()
            .context("tree entry contain invalid UTF-8")?;
        let Some((mode, name)) = mode_and_name.split_once(' ') else {
            bail!("invalid tree entry format");
        };
        object
            .reader
            .read_exact(&mut hash_buf)
            .context("invalid tree entry format")?;
        let kind = ObjectKind::from_mode(mode)?;
        let out_entry = if name_only {
            format!("{name}\n")
        } else {
            format!(
                "{:0>6} {} {}    {name}\n",
                mode,
                kind.to_str(),
                hex::encode(hash_buf)
            )
        };
        sout.write_all(out_entry.as_bytes())
            .context("write to stdout failed")?;
    }
    Ok(())
}
