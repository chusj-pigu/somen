use std::io::{self, Read};

use noodles_bgzf as bgzf;

use crate::io::reader::num::{read_i32_le, read_u64_le};

pub(super) fn read_intervals<R>(reader: &mut R) -> io::Result<Vec<bgzf::VirtualPosition>>
where
    R: Read,
{
    // n_intv
    let interval_count = read_interval_count(reader)?;

    (0..interval_count)
        .map(|_| {
            // ioff
            read_u64_le(reader).map(bgzf::VirtualPosition::from)
        })
        .collect()
}

fn read_interval_count<R>(reader: &mut R) -> io::Result<usize>
where
    R: Read,
{
    read_i32_le(reader)
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_intervals() -> io::Result<()> {
        let src = [
            0x00, 0x00, 0x00, 0x00, // n_intv = 0
        ];
        assert!(read_intervals(&mut &src[..])?.is_empty());

        let src = [
            0x01, 0x00, 0x00, 0x00, // n_intv = 1
            0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ioffset[0] = 8
        ];
        assert_eq!(
            read_intervals(&mut &src[..])?,
            vec![bgzf::VirtualPosition::from(8)]
        );

        let src = [
            0xff, 0xff, 0xff, 0xff, // n_intv = -1
        ];
        assert!(matches!(
            read_intervals(&mut &src[..]),
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));

        Ok(())
    }

    #[test]
    fn test_read_interval_count() -> io::Result<()> {
        assert_eq!(read_interval_count(&mut &[0x00, 0x00, 0x00, 0x00][..])?, 0);
        assert_eq!(read_interval_count(&mut &[0x01, 0x00, 0x00, 0x00][..])?, 1);
        assert_eq!(
            read_interval_count(&mut &[0x00, 0x01, 0x00, 0x00][..])?,
            256
        );

        assert!(matches!(
            read_interval_count(&mut io::empty()),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));
        assert!(matches!(
            read_interval_count(&mut &[0xff, 0xff, 0xff, 0xff][..]),
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));

        Ok(())
    }
}
