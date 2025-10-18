use anyhow::{Context, Result, anyhow};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs::{File, create_dir_all, rename};
use std::io::{Write, copy, sink};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub fn git_hash_object(file: PathBuf, write: bool) -> Result<()> {
    let hash = if write {
        let mut tmp_file = NamedTempFile::new()?;
        let hash = write_blob(file.as_path(), &mut tmp_file)?;
        create_dir_all(format!(".git/objects/{}", &hash[..2]))
            .context("creating git object directory")?;
        rename(
            tmp_file,
            format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
        )
        .context("renaming object")?;
        hash
    } else {
        write_blob(file.as_path(), &mut sink())?
    };
    println!("{hash}");
    Ok(())
}

fn write_blob<W: Write>(file: &Path, writer: W) -> Result<String> {
    let mut file =
        File::open(file).map_err(|e| anyhow!("error reading provided file path: {e}"))?;
    let metadata = file.metadata().context("error getting metadata")?;
    let size = metadata.len();
    let encoder = ZlibEncoder::new(writer, Compression::default());
    let mut hash_writer = HashWriter {
        writer: encoder,
        hasher: Sha1::new(),
    };
    write!(hash_writer, "blob {}\0", size)?;
    copy(&mut file, &mut hash_writer)?;
    let _compressed = hash_writer.writer.finish()?;
    let hash = hash_writer.hasher.finalize();
    Ok(hex::encode(hash))
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
