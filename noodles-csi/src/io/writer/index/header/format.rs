use std::io::{self, Write};

use crate::{
    binning_index::index::header::{Format, format::CoordinateSystem},
    io::writer::num::write_i32_le,
};

pub(super) fn write_format<W>(writer: &mut W, format: Format) -> io::Result<()>
where
    W: Write,
{
    let n = encode(format);
    write_i32_le(writer, n)
}

fn encode(format: Format) -> i32 {
    const COORDINATE_SYSTEM_SHIFT: usize = 16;

    const ONE_BASED_COORDINATE_SYSTEM_VALUE: i32 = 0x00;
    const ZERO_BASED_COORDINATE_SYSTEM_VALUE: i32 = 0x01;

    const GENERIC_FORMAT_VALUE: i32 = 0x00;
    const SAM_FORMAT_VALUE: i32 = 0x01;
    const VCF_FORMAT_VALUE: i32 = 0x02;

    match format {
        Format::Generic(CoordinateSystem::Gff) => {
            ONE_BASED_COORDINATE_SYSTEM_VALUE << COORDINATE_SYSTEM_SHIFT | GENERIC_FORMAT_VALUE
        }
        Format::Generic(CoordinateSystem::Bed) => {
            ZERO_BASED_COORDINATE_SYSTEM_VALUE << COORDINATE_SYSTEM_SHIFT | GENERIC_FORMAT_VALUE
        }
        Format::Sam => ONE_BASED_COORDINATE_SYSTEM_VALUE | SAM_FORMAT_VALUE,
        Format::Vcf => ONE_BASED_COORDINATE_SYSTEM_VALUE | VCF_FORMAT_VALUE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode(Format::Generic(CoordinateSystem::Bed)), 0x00010000);
        assert_eq!(encode(Format::Generic(CoordinateSystem::Gff)), 0x00000000);
        assert_eq!(encode(Format::Sam), 0x00000001);
        assert_eq!(encode(Format::Vcf), 0x00000002);
    }
}
