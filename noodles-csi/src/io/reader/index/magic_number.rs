use std::{
    error, fmt,
    io::{self, Read},
};

use crate::io::MAGIC_NUMBER;

#[derive(Debug)]
pub enum ReadError {
    /// I/O error.
    Io(io::Error),
    /// The input is invalid.
    Invalid([u8; MAGIC_NUMBER.len()]),
}

impl error::Error for ReadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Invalid(_) => None,
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => write!(f, "I/O error"),
            Self::Invalid(actual) => write!(f, "expected {MAGIC_NUMBER:#04x?}, got {actual:#04x?}"),
        }
    }
}

pub(super) fn read_magic_number<R>(reader: &mut R) -> Result<(), ReadError>
where
    R: Read,
{
    let mut buf = [0; MAGIC_NUMBER.len()];
    reader.read_exact(&mut buf).map_err(ReadError::Io)?;

    if buf == MAGIC_NUMBER {
        Ok(())
    } else {
        Err(ReadError::Invalid(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_magic_number() {
        assert!(read_magic_number(&mut &b"CSI\x01"[..]).is_ok());

        assert!(matches!(
            read_magic_number(&mut &[][..]),
            Err(ReadError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        assert!(matches!(
            read_magic_number(&mut &b"MThd"[..]),
            Err(ReadError::Invalid(buf)) if buf == *b"MThd"
        ));
    }
}
