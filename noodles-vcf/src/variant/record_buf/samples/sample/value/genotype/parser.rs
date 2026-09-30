use std::{error, fmt};

use memchr::{memchr, memchr2};

use super::{Allele, Genotype, allele};
use crate::variant::record::samples::series::value::genotype::Phasing;

/// An error returned when a raw VCF record genotype value fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// An allele is invalid.
    InvalidAllele(allele::ParseError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Empty => None,
            Self::InvalidAllele(e) => Some(e),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty input"),
            Self::InvalidAllele(_) => f.write_str("invalid allele"),
        }
    }
}

pub(super) fn parse(mut s: &str) -> Result<Genotype, ParseError> {
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    let first_allele = parse_first_allele(&mut s).map_err(ParseError::InvalidAllele)?;
    let mut alleles = vec![first_allele];

    while !s.is_empty() {
        let allele = parse_allele(&mut s).map_err(ParseError::InvalidAllele)?;
        alleles.push(allele);
    }

    Ok(Genotype(alleles))
}

fn next_allele<'a>(s: &mut &'a str) -> &'a str {
    let src = &s.as_bytes()[1..];
    let i = memchr2(b'|', b'/', src).map(|i| i + 1).unwrap_or(s.len());
    let (buf, rest) = s.split_at(i);
    *s = rest;
    buf
}

fn parse_first_allele(s: &mut &str) -> Result<Allele, allele::ParseError> {
    use super::allele::{parse_phasing, parse_position};

    let mut buf = next_allele(s);

    let phasing = if let Some(src) = split_off_explicit_phasing(&mut buf) {
        parse_phasing(src)?
    } else if is_implicitly_unphased(s) {
        Phasing::Unphased
    } else {
        Phasing::Phased
    };

    let position = parse_position(buf)?;

    Ok(Allele::new(position, phasing))
}

fn split_off_explicit_phasing<'a>(src: &mut &'a str) -> Option<&'a str> {
    if src.starts_with(['|', '/']) {
        let (buf, rest) = src.split_at(1);
        *src = rest;
        Some(buf)
    } else {
        None
    }
}

// § 1.6.2 "Genotype fields" (2026-02-25): "The first phasing indicator may be omitted and is
// implicitly defined as / if any phasing indicators are /..."
fn is_implicitly_unphased(src: &str) -> bool {
    memchr(b'/', src.as_bytes()).is_some()
}

fn parse_allele(src: &mut &str) -> Result<Allele, allele::ParseError> {
    let buf = next_allele(src);
    buf.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_off_explicit_phasing() {
        let mut src = "|0";
        assert_eq!(split_off_explicit_phasing(&mut src), Some("|"));
        assert_eq!(src, "0");

        let mut src = "/0";
        assert_eq!(split_off_explicit_phasing(&mut src), Some("/"));
        assert_eq!(src, "0");

        let mut src = "0";
        assert!(split_off_explicit_phasing(&mut src).is_none());
        assert_eq!(src, "0");
    }

    #[test]
    fn test_is_implicitly_unphased() {
        assert!(!is_implicitly_unphased("0"));
        assert!(!is_implicitly_unphased("0|0"));
        assert!(is_implicitly_unphased("0/0"));
    }

    #[test]
    fn test_next_allele() {
        let mut src = "0";
        assert_eq!(next_allele(&mut src), "0");
        assert!(src.is_empty());

        let mut src = "|0/0";
        assert_eq!(next_allele(&mut src), "|0");
        assert_eq!(next_allele(&mut src), "/0");
        assert!(src.is_empty());
    }
}
