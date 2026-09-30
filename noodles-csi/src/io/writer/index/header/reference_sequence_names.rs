use std::io::{self, Write};

use bstr::{BStr, ByteSlice};

use crate::{binning_index::index::header::ReferenceSequenceNames, io::writer::num::write_i32_le};

const NUL: u8 = 0x00;

pub(super) fn write_reference_sequence_names<W>(
    writer: &mut W,
    reference_sequence_names: &ReferenceSequenceNames,
) -> io::Result<()>
where
    W: Write,
{
    write_reference_sequence_names_size(writer, reference_sequence_names)?;

    for reference_sequence_name in reference_sequence_names {
        write_reference_sequence_name(writer, reference_sequence_name.as_ref())?;
    }

    Ok(())
}

fn write_reference_sequence_names_size<W>(
    writer: &mut W,
    reference_sequence_names: &ReferenceSequenceNames,
) -> io::Result<()>
where
    W: Write,
{
    let len = size_of_reference_sequence_names(reference_sequence_names);
    let n = i32::try_from(len).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    write_i32_le(writer, n)
}

fn size_of_reference_sequence_names(reference_sequence_names: &ReferenceSequenceNames) -> usize {
    const SIZE_OF_NUL: usize = size_of::<u8>();

    reference_sequence_names
        .iter()
        .map(|n| n.len() + SIZE_OF_NUL)
        .sum::<usize>()
}

fn write_reference_sequence_name<W>(
    writer: &mut W,
    reference_sequence_name: &BStr,
) -> io::Result<()>
where
    W: Write,
{
    if !is_valid(reference_sequence_name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid reference sequence name",
        ));
    }

    writer.write_all(reference_sequence_name)?;
    writer.write_all(&[NUL])?;

    Ok(())
}

fn is_valid(s: &BStr) -> bool {
    s.find_byte(NUL).is_none()
}

#[cfg(test)]
mod tests {
    use bstr::BString;

    use super::*;

    #[test]
    fn test_write_reference_sequence_names() -> io::Result<()> {
        let mut buf = Vec::new();

        let reference_sequence_names = ReferenceSequenceNames::default();
        buf.clear();
        write_reference_sequence_names(&mut buf, &reference_sequence_names)?;
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]); // l_nm = 0

        let reference_sequence_names = [BString::from("sq0")].into_iter().collect();
        buf.clear();
        write_reference_sequence_names(&mut buf, &reference_sequence_names)?;
        assert_eq!(
            buf,
            [
                0x04, 0x00, 0x00, 0x00, // l_nm = 4
                b's', b'q', b'0', 0x00, // names[0] = "sq0"
            ]
        );

        let reference_sequence_names = [BString::from("sq0"), BString::from("sq1")]
            .into_iter()
            .collect();
        buf.clear();
        write_reference_sequence_names(&mut buf, &reference_sequence_names)?;
        assert_eq!(
            buf,
            [
                0x08, 0x00, 0x00, 0x00, // l_nm = 8
                b's', b'q', b'0', 0x00, // names[0] = "sq0"
                b's', b'q', b'1', 0x00, // names[1] = "sq1"
            ]
        );

        let reference_sequence_names = [BString::from("sq\x000")].into_iter().collect();
        buf.clear();
        assert!(matches!(
            write_reference_sequence_names(&mut buf, &reference_sequence_names),
            Err(e) if e.kind() == io::ErrorKind::InvalidInput
        ));

        Ok(())
    }

    #[test]
    fn test_size_of_reference_sequence_names() {
        let reference_sequence_names = ReferenceSequenceNames::default();
        assert_eq!(
            size_of_reference_sequence_names(&reference_sequence_names),
            0
        );

        let reference_sequence_names = [BString::from("sq0")].into_iter().collect();
        assert_eq!(
            size_of_reference_sequence_names(&reference_sequence_names),
            4
        );

        let reference_sequence_names = [BString::from("sq0"), BString::from("sq1")]
            .into_iter()
            .collect();
        assert_eq!(
            size_of_reference_sequence_names(&reference_sequence_names),
            8
        );
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid(b"sq0".as_bstr()));
        assert!(!is_valid(b"sq\x000".as_bstr()));
    }
}
