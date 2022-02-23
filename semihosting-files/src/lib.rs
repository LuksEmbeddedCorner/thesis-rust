#![no_std]
#![allow(clippy::result_unit_err)]
#![warn(missing_docs)]

//! This crate allows to access the files of a host machine while semihosting.
//! It should work for all Cortex-M processors.
//!
//! For examples on how to use this crate, check out the examples on [github](https://github.com/LuksEmbeddedCorner/semihosting-files/tree/master/examples).
//!
//! Click [here](https://developer.arm.com/documentation/dui0471/m/what-is-semihosting-/what-is-semihosting-?lang=en) for a reference to the underlying API.

#[macro_use]
extern crate cortex_m_semihosting;

use core::mem;

use cortex_m_semihosting::nr::open;

use cstr_core::CStr;

// FIXME Implement Reader/Writer  when a working core_io version exists
// #[cfg(feature="core_io")]
// mod core_io;

/// A reference to an open file on the host system.
///
/// Depending on the options that it was opened with, files can be read and/or written.
///
/// Files are auromatically closed when they are dropped. If an error occurs while dropping, the error is silently swallowed.
/// Use [`close`](File::close) if these errors should be explicitly handled.
///
/// Because all semihosting operations are very slow, buffering reads and writes might be beneficial.
///
/// As the semihosting operations don't return a specific error when they fail, most methods just return a `Result<_, ()>`.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct File {
    handle: isize,
}

/// This enum determines how a file should be opened.
///
/// For more informations on the open modes, see the c-function `fopen`.
#[repr(usize)]
#[allow(missing_docs)]
pub enum FileOpenMode {
    Read = open::R,
    ReadBinary = open::R_BINARY,
    ReadWrite = open::RW,
    ReadWriteAppend = open::RW_APPEND,
    ReadWriteAppendBinary = open::RW_APPEND_BINARY,
    ReadWriteTruncate = open::RW_TRUNC,
    ReadWriteTruncateBinary = open::RW_TRUNC_BINARY,
    WriteAppend = open::W_APPEND,
    WriteAppendBinary = open::W_APPEND_BINARY,
    WriteTruncate = open::W_TRUNC,
    WriteTruncateBinary = open::W_TRUNC_BINARY,
}

/// Specifies the cursor position for [`File::seek`]
pub enum SeekFrom {
    /// Sets the cursor to the specified number of bytes, starting from the beginning of the file
    ///
    /// The provided number must be ranging from 0 to the length of the file
    Start(usize),
    /// Sets the cursor to the length of the file, plus the specified amount.
    ///
    /// Note that the cursor cannot be set to a absolute position greater than the length of the file,
    /// so the relative position provided must be number ranging from the negative length of the file to 0.
    End(isize),
}

/// An Error that occured during seeking
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SeekError {
    /// The specified position was outside the boundary of the file
    PositionOutOfBounds,
    /// An unknown error occured while reading the length of the file, or setting the cursor.
    Unknown,
}

impl File {
    /// Opens the file with the given mode.
    pub fn open(path: &CStr, mode: FileOpenMode) -> Result<File, ()> {
        let handle = unsafe { syscall!(OPEN, path.as_ptr(), mode, path.to_bytes().len()) } as isize;

        if handle == -1 {
            return Err(());
        }

        Ok(File { handle })
    }

    /// Tries to write all of buffers bytes into the file, and returns the number of bytes that were actually written.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let not_written = unsafe { syscall!(WRITE, self.handle, buf.as_ptr(), buf.len()) };

        if not_written > buf.len() {
            return Err(());
        }

        Ok(buf.len() - not_written)
    }

    /// Closes the file.
    ///
    /// The file is also closed when it the file is dropped.
    /// However this method is more explicit and allows to check if an error happenend while closing the file.
    ///
    /// If an error occured, the unclosed file is returned.
    pub fn close(self) -> Result<(), File> {
        match self.close_internal() {
            Ok(()) => {
                // Drop should not be called again,
                // because the file would be closed twice
                mem::forget(self);

                Ok(())
            }
            Err(()) => Err(self),
        }
    }

    fn close_internal(&self) -> Result<(), ()> {
        let success = unsafe { syscall!(CLOSE, self.handle) };

        if success != 0 {
            return Err(());
        }

        Ok(())
    }

    /// Retrieves the total number of bytes of the file
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<usize, ()> {
        let length = unsafe { syscall!(FLEN, self.handle) };

        if (length as isize) < 0 {
            return Err(());
        }

        Ok(length)
    }

    /// Tries to read `buf.len()` bytes from the file and returns the number of bytes that was read into the buffer.
    ///
    /// The result `Ok(0usize)` suggests that EOF has been reached.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let not_read = unsafe { syscall!(READ, self.handle, buf.as_mut_ptr(), buf.len()) };

        if not_read > buf.len() {
            return Err(());
        }

        Ok(buf.len() - not_read)
    }

    /// Sets the read/write-cursor to the specified position
    /// This actually consists of two semihosting operations:
    /// One to get the length, and another to set the cursor
    ///
    /// If you want to set the cursor to the beginning of the file, use [`rewind`](File::rewind) instead.
    ///
    /// see also: [`seek_unchecked`](File::seek_unchecked)
    pub fn seek(&mut self, from: SeekFrom) -> Result<(), SeekError> {
        let length = self.len().map_err(|()| SeekError::Unknown)?;

        let pos = match from {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => length.wrapping_add(offset as usize),
        };

        if pos > length {
            return Err(SeekError::PositionOutOfBounds);
        }

        // Safety: pos has been checked, it contains a valid value
        let result = unsafe { self.seek_unchecked(pos) };

        result.map_err(|()| SeekError::Unknown)
    }

    /// Sets the position of the cursor.
    ///
    /// The position is specified relative to the beginning of the file.
    ///
    /// See also [`seek`](File::seek) for a safe, and slightly more ergonomic way to set the cursor.
    ///
    /// # Safety
    ///
    /// The position must lie inside the extend of the file.
    /// Otherwise, the behaviour is undefined.
    pub unsafe fn seek_unchecked(&mut self, pos: usize) -> Result<(), ()> {
        let result = syscall!(SEEK, self.handle, pos) as isize;

        if result < 0 {
            return Err(());
        }

        Ok(())
    }

    /// Sets the cursor to the beginning of the file.
    ///
    /// In comparison to [`seek`](File::seek), this method does not need to check the length of the file or the provided index.
    pub fn rewind(&mut self) -> Result<(), ()> {
        // Safety: 0 is always a valid position
        unsafe { self.seek_unchecked(0) }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // Errors are ignored, just like in std::fs::File
        let _ = self.close_internal();
    }
}
