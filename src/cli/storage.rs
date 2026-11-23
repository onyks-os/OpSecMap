// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Reading and writing map files on disk, for `save` and `open`.
//!
//! Reading refuses any file larger than [`MAX_FILE_BYTES`]. Writing goes through
//! a temporary file in the same folder, so an interrupted save never leaves a
//! half-written map. The file is plain text and is not encrypted.

use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// The largest map file that [`read_map_file`] accepts: 1 MiB.
///
/// The limit stops a huge file, chosen by mistake, from filling memory before
/// it is even parsed.
pub(crate) const MAX_FILE_BYTES: u64 = 1_048_576; // 1MiB

/// Reads the whole file at `path` as text.
///
/// # Errors
///
/// Returns [`StorageError::Io`] if the file cannot be opened or read, or is not
/// valid UTF-8, and [`StorageError::TooLarge`] if it is larger than
/// [`MAX_FILE_BYTES`].
pub(crate) fn read_map_file(path: &Path) -> Result<String, StorageError> {
    let file = File::open(path).map_err(StorageError::Io)?;
    let mut reader = file.take(MAX_FILE_BYTES + 1);
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buffer).map_err(StorageError::Io)?;
    if buffer.len() as u64 > MAX_FILE_BYTES {
        return Err(StorageError::TooLarge);
    }
    String::from_utf8(buffer).map_err(|error| {
        StorageError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    })
}

/// Writes `contents` to `path`, replacing any existing file in a single step.
///
/// The text is first written to `path` with `.tmp` appended and flushed to disk;
/// the temporary file is then renamed over `path`. If a step fails, the
/// temporary file is removed and the previous file at `path`, if any, is left
/// unchanged.
///
/// # Errors
///
/// Returns [`StorageError::Io`] if the temporary file cannot be created,
/// written, flushed or renamed.
pub(crate) fn write_map_file(path: &Path, contents: &str) -> Result<(), StorageError> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    if let Err(error) = write_then_rename(&tmp, path, contents) {
        let _ = fs::remove_file(&tmp);
        return Err(StorageError::Io(error));
    }
    Ok(())
}

/// Creates `tmp`, writes `contents` to it, flushes it to disk and renames it
/// to `path`.
///
/// # Errors
///
/// Returns the first I/O error. It does not remove `tmp`; [`write_map_file`]
/// does that.
fn write_then_rename(tmp: &Path, path: &Path, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(tmp)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    fs::rename(tmp, path)?;
    Ok(())
}

/// An error while reading or writing a map file.
#[derive(Debug)]
pub(crate) enum StorageError {
    /// The operating system reported an error, such as a missing file or a
    /// denied permission.
    Io(std::io::Error),
    /// The file is larger than [`MAX_FILE_BYTES`].
    TooLarge,
}

/// Describes the error in one lowercase line, for the CLI's error output.
impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(error) => write!(f, "cannot access the map file: {error}"),
            StorageError::TooLarge => {
                write!(f, "map file is larger than {MAX_FILE_BYTES} bytes")
            }
        }
    }
}
