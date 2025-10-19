use crate::object_read::Object;
use anyhow::Result;
use std::io::sink;
use std::path::Path;

/// In Git, each file is stored as a *blob object*.
///
/// The blob’s raw (uncompressed) format is:
///     "blob <size>\0<content of file>"
///
/// Steps:
/// 1. Compute the SHA-1 (or SHA-256, depending on repo format) hash
///    of the uncompressed data: `"blob <size>\0<content>"`.
///
/// 2. Hex-encode this hash — this becomes the blob’s *object ID*.
///
/// 3. Compress the uncompressed data using zlib.
///
/// 4. Write the compressed bytes to:
///    .git/objects/<first 2 hex chars of hash>/<remaining 38 chars>
///
/// Example:
///   File content: "hello world\n"
///   Uncompressed form: "blob 12\0hello world\n"
///   SHA-1 hash: 557db03de997c86a4a028e1ebd3a1ceb225be238
///   Stored at:
///     .git/objects/55/7db03de997c86a4a028e1ebd3a1ceb225be238
///
/// Note: Git only stores the file *contents* in the blob —
///       file names and permissions are stored in *tree objects*.
///
pub fn git_hash_object(file: &Path, write: bool) -> Result<[u8; 20]> {
    if write {
        Object::from_blob_file(file)?.write_as_object()
    } else {
        Object::from_blob_file(file)?.write(sink())
    }
}
