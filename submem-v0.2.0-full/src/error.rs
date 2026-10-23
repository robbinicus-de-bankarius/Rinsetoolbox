//! Recoverable error handling modeled after The Rust Book Ch. 9.2 (`Result<T, E>`, `?` operator)
//! and Google's Comprehensive Rust (`Error Handling: Try Conversions & From`).

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SubmemError {
    Io(io::Error),
    InvalidMagic([u8; 4]),
    UnsupportedVersion(u8),
    ChecksumMismatch {
        turn_id: u32,
        expected: u32,
        actual: u32,
    },
    TruncatedFrame {
        offset: u64,
        detail: &'static str,
    },
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidFlagBits(u8),
    InvalidArgument(String),
}

impl fmt::Display for SubmemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubmemError::Io(err) => write!(f, "I/O error: {err}"),
            SubmemError::InvalidMagic(m) => {
                write!(f, "Invalid SMEM magic header: {m:?} (expected b\"SMEM\")")
            }
            SubmemError::UnsupportedVersion(v) => {
                write!(f, "Unsupported SMEM binary version: {v}")
            }
            SubmemError::ChecksumMismatch {
                turn_id,
                expected,
                actual,
            } => write!(
                f,
                "CRC32 checksum mismatch on turn #{turn_id}: expected {expected:#010x}, got {actual:#010x}"
            ),
            SubmemError::TruncatedFrame { offset, detail } => {
                write!(f, "Truncated binary frame at byte offset {offset}: {detail}")
            }
            SubmemError::InvalidUtf8(err) => write!(f, "UTF-8 decode error: {err}"),
            SubmemError::InvalidFlagBits(b) => write!(f, "Invalid packed flag byte: {b:#010b}"),
            SubmemError::InvalidArgument(msg) => write!(f, "Invalid argument: {msg}"),
        }
    }
}

impl std::error::Error for SubmemError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SubmemError::Io(err) => Some(err),
            SubmemError::InvalidUtf8(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for SubmemError {
    fn from(err: io::Error) -> Self {
        SubmemError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for SubmemError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        SubmemError::InvalidUtf8(err)
    }
}

pub type Result<T> = std::result::Result<T, SubmemError>;
