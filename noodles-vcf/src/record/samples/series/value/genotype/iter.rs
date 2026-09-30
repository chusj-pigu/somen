use std::io;

use memchr::{memchr, memchr2};

use crate::variant::record::samples::series::value::genotype::Phasing;

pub(super) struct Iter<'a> {
    src: &'a str,
    is_first_allele: bool,
}

impl<'a> Iter<'a> {
    pub(super) fn new(src: &'a str) -> Self {
        Self {
            src,
            is_first_allele: true,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = io::Result<(Option<usize>, Phasing)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            if self.is_first_allele {
                self.is_first_allele = false;
                Some(Err(io::Error::from(io::ErrorKind::UnexpectedEof)))
            } else {
                None
            }
        } else if self.is_first_allele {
            self.is_first_allele = false;
            Some(parse_first_allele(&mut self.src))
        } else {
            Some(parse_allele(&mut self.src))
        }
    }
}

fn parse_first_allele(src: &mut &str) -> io::Result<(Option<usize>, Phasing)> {
    let mut buf = next_allele(src);

    let phasing = if let Some(s) = split_off_explicit_phasing(&mut buf) {
        parse_phasing(s)?
    } else if is_implicitly_unphased(src) {
        Phasing::Unphased
    } else {
        Phasing::Phased
    };

    let position = parse_position(buf)?;

    Ok((position, phasing))
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

fn parse_allele(src: &mut &str) -> io::Result<(Option<usize>, Phasing)> {
    let buf = next_allele(src);

    let phasing = parse_phasing(&buf[..1])?;
    let position = parse_position(&buf[1..])?;

    Ok((position, phasing))
}

fn next_allele<'a>(src: &mut &'a str) -> &'a str {
    let s = &src.as_bytes()[1..];
    let i = memchr2(b'|', b'/', s).map(|i| i + 1).unwrap_or(src.len());
    let (buf, rest) = src.split_at(i);
    *src = rest;
    buf
}

fn parse_phasing(src: &str) -> io::Result<Phasing> {
    const PHASED: &str = "|";
    const UNPHASED: &str = "/";

    match src {
        PHASED => Ok(Phasing::Phased),
        UNPHASED => Ok(Phasing::Unphased),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid phasing indicator",
        )),
    }
}

fn parse_position(src: &str) -> io::Result<Option<usize>> {
    const MISSING: &str = ".";

    match src {
        MISSING => Ok(None),
        _ => src
            .parse()
            .map(Some)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() -> io::Result<()> {
        fn t(src: &str, expected: &[(Option<usize>, Phasing)]) -> io::Result<()> {
            let iter = Iter::new(src);
            let actual: Vec<_> = iter.collect::<io::Result<_>>()?;
            assert_eq!(actual, expected);
            Ok(())
        }

        t(
            "0|0",
            &[(Some(0), Phasing::Phased), (Some(0), Phasing::Phased)],
        )?;
        t(
            "0/1",
            &[(Some(0), Phasing::Unphased), (Some(1), Phasing::Unphased)],
        )?;
        t(
            "|0/1",
            &[(Some(0), Phasing::Phased), (Some(1), Phasing::Unphased)],
        )?;
        t(
            "|1/2|3",
            &[
                (Some(1), Phasing::Phased),
                (Some(2), Phasing::Unphased),
                (Some(3), Phasing::Phased),
            ],
        )?;
        t(
            "./.",
            &[(None, Phasing::Unphased), (None, Phasing::Unphased)],
        )?;

        let iter = Iter::new("");
        assert!(matches!(
            iter.collect::<io::Result<Vec<_>>>(),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        Ok(())
    }

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
