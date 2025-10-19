use crate::ls_tree::git_read_tree_content;
use crate::object_read::{Object, ObjectKind};
use anyhow::{Result, ensure};
use std::io::{Read, copy, stdout};

pub fn git_cat_file(pretty_print: bool, object_hash: &str) -> Result<()> {
    ensure!(
        pretty_print,
        "type or -p need to be passed and we don't support type at the moment"
    );
    let object = Object::read_git_object(object_hash)?;
    match object.kind {
        ObjectKind::Blob | ObjectKind::Commit => {
            let mut sout = stdout().lock();
            // Read max of the size from the file.
            // Protect against zipbomb.
            // Ignore the content after the expected size.
            let a_size = copy(&mut object.reader.take(object.expected_size), &mut sout)?;
            ensure!(
                a_size == object.expected_size,
                "object size mismatch, expected {}, got {}",
                object.expected_size,
                a_size
            );
        }
        ObjectKind::Tree => {
            git_read_tree_content(object, false)?;
        }
    }
    Ok(())
}
