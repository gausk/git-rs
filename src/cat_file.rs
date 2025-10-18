use crate::ObjectKind;
use anyhow::{Context, Result, anyhow, bail, ensure};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, copy, stdout};

pub fn git_cat_file(pretty_print: bool, hash_object: String) -> Result<()> {
    ensure!(
        pretty_print,
        "type or -p need to be passed and we don't support type at the moment"
    );
    if hash_object.len() < 3 {
        bail!("Hash objects len must be at least 3");
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(format!(".git/objects/{}", &hash_object[..2]))
        .map_err(|e| anyhow!("error reading .git/objects directory: {}", e))?
    {
        let entry = entry?;
        let path = entry.path();
        if entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("file name conversion error"))?
            .starts_with(&hash_object[2..])
            && entry.file_type()?.is_file()
        {
            files.push(path);
        }
    }
    if files.is_empty() {
        bail!("No objects found");
    } else if files.len() > 1 {
        bail!("Multiple objects found: {}", files.len());
    }
    let file = File::open(&files[0])?;
    let decoder = ZlibDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let mut buf = Vec::new();
    reader
        .read_until(0, &mut buf)
        .context("failed to read header")?;
    let header = CStr::from_bytes_with_nul(&buf).context("header is in invalid format")?;
    let header = header.to_str().context("header is not valid UTF-8")?;
    let Some((kind, size)) = header.split_once(' ') else {
        bail!("header is in invalid format");
    };
    let kind = match kind {
        "blob" => ObjectKind::Blob,
        _ => bail!("unknown object kind: {}", kind),
    };
    let size = size.parse::<u64>().context("object size isn't a number")?;
    match kind {
        ObjectKind::Blob => {
            let mut sout = stdout().lock();
            // Read max of the size from the file.
            // Protect against zipbomb.
            // Ignore the content after the expected size.
            let a_size = copy(&mut reader.take(size), &mut sout)?;
            ensure!(
                a_size == size,
                "object size mismatch, expected {}, got {}",
                size,
                a_size
            );
        }
    }
    Ok(())
}
