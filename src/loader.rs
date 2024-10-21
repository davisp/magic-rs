use std::path::Path;

use thiserror::Error;

use crate::magic::{Magic, MagicError};
use crate::structs::MagicMap;
//use crate::structs::MagicMap;
use crate::traits::ReadLittleEndian;

/// The magic number for libmagic databases
const MAGIC_CONSTANT: u32 = 0xF11E041C;

/// Magic Database Version
const MAGIC_DATABASE_VERSION: u32 = 19;

/// A `struct magic` from libmagic is 432 bytes
pub(crate) const MAGIC_SIZE: usize = 432;

/// The number of Magic sets. I have no idea why this is 2... yet.
const MAGIC_SETS: usize = 2;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Error reading file '{0}': {1}")]
    Io(String, std::io::Error),
    #[error("Database length {0} is not a multiple of MAGIC_SIZE ({1}).")]
    InvalidBufferLength(usize, usize),
    #[error(
        "Invalid number of records found: {0} expected {1} based on file size."
    )]
    InvalidDatabaseRecordCount(usize, usize),
    #[error("Invalid database version: {0}. Only version {1} is supported.")]
    InvalidDatabaseVersion(u32, u32),
    #[error("The magic database is not little endian.")]
    InvalidEndianness,
    #[error("The magic database has an invalid magic constant: {0:#08x} expected: {1:#08x}")]
    InvalidMagicConstant(u32, u32),
    #[error("Database only contains room for {0} records, 3 are required.")]
    InvalidRecordCount(usize),
    #[error("Error deserializing Magic entry: {0}")]
    Magic(#[from] MagicError),
}

type Result<T> = std::result::Result<T, LoaderError>;

pub fn load_db<P: AsRef<Path>>(path: P) -> Result<MagicMap> {
    let bytes = std::fs::read(&path)
        .map_err(|e| LoaderError::Io(path.as_ref().display().to_string(), e))?;
    load_db_impl(bytes)
}

fn load_db_impl(bytes: Vec<u8>) -> Result<MagicMap> {
    let num_records = bytes.len() / MAGIC_SIZE;
    if num_records * MAGIC_SIZE != bytes.len() {
        return Err(LoaderError::InvalidBufferLength(bytes.len(), MAGIC_SIZE));
    }

    // N.B., the +1 here is because libmagic reserves the first "entry" to just
    // be a few pieces of metadata. I have no idea yet why there are exaclt two
    // magic sets.
    if num_records < MAGIC_SETS + 1 {
        return Err(LoaderError::InvalidRecordCount(num_records));
    }

    // At this point we know that bytes.len() is at least 3 * MAGIC_SIZE
    // bytes and that these metadata values are all in the first MAGIC_SIZE
    // bytes of the buffer which means we can elide length checks here.

    // Check that the magic number is correct.
    let magic = u32::read_le(&bytes);
    if magic != MAGIC_CONSTANT {
        if u32::from_be(magic) == MAGIC_CONSTANT {
            return Err(LoaderError::InvalidEndianness);
        }
        return Err(LoaderError::InvalidMagicConstant(magic, MAGIC_CONSTANT));
    }

    let version = u32::read_le(&bytes[4..]);
    if version != MAGIC_DATABASE_VERSION {
        return Err(LoaderError::InvalidDatabaseVersion(
            version,
            MAGIC_DATABASE_VERSION,
        ));
    }

    // N.B., libmagic does this next part as a loop with some weird logic
    // when its a defined constant of 2 for the format. So I'm skipping the loop.
    let left_num_records = u32::read_le(&bytes[8..]);
    let right_num_records = u32::read_le(&bytes[12..]);

    if (1 + left_num_records + right_num_records) as usize != num_records {
        return Err(LoaderError::InvalidDatabaseRecordCount(
            (left_num_records + right_num_records) as usize,
            num_records,
        ));
    }

    // At this point, libmagic just creates two arrays of Magic structs by
    // reinterpreting the input bytes as two arrays split at left_num_records.
    // Rather than slapping a big unsafe block here to do such a thing and hope
    // for the best with struct alignments (yes, I know about repr(C), still
    // not doing it) I just parse each record into a vec and then split that
    // vec into two.
    let record_bytes = &bytes[MAGIC_SIZE..];
    let iter = record_bytes.chunks_exact(MAGIC_SIZE);
    assert!(iter.remainder().is_empty());

    let mut records = iter
        .map(Magic::from_bytes)
        .collect::<std::result::Result<Vec<_>, MagicError>>()?;
    println!("LEFT: {} RIGHT: {}", left_num_records, right_num_records);
    let right_records = records.split_off(left_num_records as usize);
    records.shrink_to_fit();

    Ok(MagicMap {
        left: records,
        right: right_records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_db() -> Result<()> {
        let map = load_db("data/magic.mgc")?;

        for m in map.left {
            println!("< {:#?}", m);
        }

        for m in map.right {
            println!("> {:#?}", m);
        }

        Ok(())
    }
}
