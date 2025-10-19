use crate::object_read::{Object, ObjectKind};
use anyhow::{Context, Result, anyhow};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs::{File, create_dir_all, rename};
use std::io::{Read, Write, copy};
use std::path::Path;
use tempfile::NamedTempFile;

impl Object<()> {
    pub(crate) fn from_blob_file(path: impl AsRef<Path>) -> Result<Object<impl Read>> {
        let reader =
            File::open(path).map_err(|e| anyhow!("error reading provided file path: {e}"))?;
        let metadata = reader.metadata().context("error getting metadata")?;
        let expected_size = metadata.len();
        Ok(Object {
            expected_size,
            kind: ObjectKind::Blob,
            reader,
        })
    }
}

impl<R> Object<R>
where
    R: Read,
{
    pub fn write(&mut self, writer: impl Write) -> Result<[u8; 20]> {
        let encoder = ZlibEncoder::new(writer, Compression::default());
        let mut hash_writer = HashWriter {
            writer: encoder,
            hasher: Sha1::new(),
        };
        write!(
            hash_writer,
            "{} {}\0",
            self.kind.to_str(),
            self.expected_size
        )?;
        copy(&mut self.reader, &mut hash_writer)?;
        let _compressed = hash_writer.writer.finish()?;
        let hash = hash_writer.hasher.finalize();
        Ok(hash.into())
    }

    pub fn write_as_object(&mut self) -> Result<[u8; 20]> {
        let mut tmp_file = NamedTempFile::new()?;
        let hash = self.write(&mut tmp_file)?;
        let hash_enc = hex::encode(hash);
        create_dir_all(format!(".git/objects/{}", &hash_enc[..2]))
            .context("creating git object directory")?;
        rename(
            tmp_file,
            format!(".git/objects/{}/{}", &hash_enc[..2], &hash_enc[2..]),
        )
        .context("renaming object")?;
        Ok(hash)
    }
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
