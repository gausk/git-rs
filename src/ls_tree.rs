use crate::object_read::{Object, ObjectKind};
use anyhow::{Context, Result, anyhow, bail};
use std::ffi::CStr;
use std::io::{BufRead, Write, stdout};

pub fn git_ls_tree(name_only: bool, tree_hash: &str) -> Result<()> {
    let object = Object::read_git_object(tree_hash)?;
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
