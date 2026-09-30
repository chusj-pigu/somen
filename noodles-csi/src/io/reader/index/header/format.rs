use std::{
    error, fmt,
    io::{self, Read},
};

use crate::{
    binning_index::index::header::{Format, format::CoordinateSystem},
    io::reader::num::read_i32_le,
};

pub fn read_format<R>(reader: &mut R) -> Result<Format, ReadError>
where
    R: Read,
{
    read_i32_le(reader).map_err(ReadError::Io).and_then(decode)
}

fn decode(n: i32) -> Result<Format, ReadError> {
    const COORDINATE_SYSTEM_SHIFT: usize = 16;
    const FORMAT_MASK: i32 = 0xffff;

    const ONE_BASED_COORDINATE_SYSTEM_VALUE: i32 = 0x00;
    const ZERO_BASED_COORDINATE_SYSTEM_VALUE: i32 = 0x01;

    const GENERIC_FORMAT_VALUE: i32 = 0x00;
    const SAM_FORMAT_VALUE: i32 = 0x01;
    const VCF_FORMAT_VALUE: i32 = 0x02;

    match (n >> COORDINATE_SYSTEM_SHIFT, n & FORMAT_MASK) {
        (ONE_BASED_COORDINATE_SYSTEM_VALUE, GENERIC_FORMAT_VALUE) => {
            Ok(Format::Generic(CoordinateSystem::Gff))
        }
        (ZERO_BASED_COORDINATE_SYSTEM_VALUE, GENERIC_FORMAT_VALUE) => {
            Ok(Format::Generic(CoordinateSystem::Bed))
        }
        (ONE_BASED_COORDINATE_SYSTEM_VALUE, SAM_FORMAT_VALUE) => Ok(Format::Sam),
        (ONE_BASED_COORDINATE_SYSTEM_VALUE, VCF_FORMAT_VALUE) => Ok(Format::Vcf),
        _ => Err(ReadError::Invalid(n)),
    }
}

/// An error returned when a CSI header format fails to be read.
#[derive(Debug)]
pub enum ReadError {
    /// An I/O error.
    Io(io::Error),
    /// The input is invalid.
    Invalid(i32),
}

impl error::Error for ReadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ReadError::Io(e) => Some(e),
            ReadError::Invalid(_) => None,
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => write!(f, "I/O error"),
            Self::Invalid(n) => write!(f, "invalid input: {n:#010x?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() -> Result<(), ReadError> {
        assert_eq!(decode(0x00000000)?, Format::Generic(CoordinateSystem::Gff));
        assert_eq!(decode(0x00010000)?, Format::Generic(CoordinateSystem::Bed));
        assert_eq!(decode(0x00000001)?, Format::Sam);
        assert_eq!(decode(0x00000002)?, Format::Vcf);

        assert!(matches!(decode(0x00020000), Err(ReadError::Invalid(_))));
        assert!(matches!(decode(0x00010001), Err(ReadError::Invalid(_))));
        assert!(matches!(decode(0x00010002), Err(ReadError::Invalid(_))));
        assert!(matches!(decode(0x00000003), Err(ReadError::Invalid(_))));

        Ok(())
    }
}
