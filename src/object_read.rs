use anyhow::{Context, Result, anyhow, bail};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
}

impl ObjectKind {
    pub fn from_str(kind: &str) -> Result<Self> {
        match kind {
            "blob" => Ok(ObjectKind::Blob),
            "tree" => Ok(ObjectKind::Tree),
            "commit" => Ok(ObjectKind::Commit),
            other => Err(anyhow!("unknown object kind: {}", other)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            ObjectKind::Blob => "blob",
            ObjectKind::Tree => "tree",
            ObjectKind::Commit => "commit",
        }
    }

    pub fn from_mode(mode: &str) -> Result<Self> {
        let mode = u32::from_str_radix(mode, 8)?;
        Ok(match mode {
            0o40000 => ObjectKind::Tree,
            0o160000 => ObjectKind::Commit,
            _ => ObjectKind::Blob,
        })
    }
}

pub struct Object<R> {
    pub(crate) reader: R,
    pub(crate) kind: ObjectKind,
    pub(crate) expected_size: u64,
}

impl Object<()> {
    pub fn read_git_object(hash: &str) -> Result<Object<impl BufRead>> {
        if hash.len() < 3 {
            bail!("Hash objects len must be at least 3");
        }
        let mut files = Vec::new();
        for entry in fs::read_dir(format!(".git/objects/{}", &hash[..2]))
            .map_err(|e| anyhow!("error reading .git/objects directory: {}", e))?
        {
            let entry = entry?;
            let path = entry.path();
            if entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("file name conversion error"))?
                .starts_with(&hash[2..])
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
        let expected_size = size.parse::<u64>().context("object size isn't a number")?;
        let kind = ObjectKind::from_str(kind)?;
        Ok(Object {
            reader,
            kind,
            expected_size,
        })
    }
}
