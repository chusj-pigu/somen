use std::{error, fmt, num};

use crate::header::{FileFormat, record::value::map::format::Number};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The input is invalid.
    Invalid(num::ParseIntError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Empty => None,
            Self::Invalid(e) => Some(e),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty input"),
            Self::Invalid(_) => f.write_str("invalid input"),
        }
    }
}

pub(super) fn parse_number(s: &str, file_format: FileFormat) -> Result<Number, ParseError> {
    const VCF_4_4: FileFormat = FileFormat::new(4, 4);

    match s {
        "" => Err(ParseError::Empty),
        "A" => Ok(Number::AlternateBases),
        "R" => Ok(Number::ReferenceAlternateBases),
        "G" => Ok(Number::Samples),
        "." => Ok(Number::Unknown),
        "P" if file_format >= VCF_4_4 => Ok(Number::Ploidy),
        _ => s.parse().map(Number::Count).map_err(ParseError::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        const VCF_4_3: FileFormat = FileFormat::new(4, 3);
        const VCF_4_4: FileFormat = FileFormat::new(4, 4);

        for file_format in [VCF_4_3, VCF_4_4] {
            assert_eq!(parse_number("1", file_format), Ok(Number::Count(1)));
            assert_eq!(parse_number("A", file_format), Ok(Number::AlternateBases));
            assert_eq!(
                parse_number("R", file_format),
                Ok(Number::ReferenceAlternateBases)
            );
            assert_eq!(parse_number("G", file_format), Ok(Number::Samples));
            assert_eq!(parse_number(".", file_format), Ok(Number::Unknown));

            assert_eq!(parse_number("", file_format), Err(ParseError::Empty));
            assert!(matches!(
                parse_number("ndls", file_format),
                Err(ParseError::Invalid(_))
            ));
        }

        assert!(matches!(
            parse_number("P", VCF_4_3),
            Err(ParseError::Invalid(_))
        ));
        assert_eq!(parse_number("P", VCF_4_4), Ok(Number::Ploidy));
    }
}
