use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use crate::binning_index::index::Header;

pub(super) async fn write_aux<W>(writer: &mut W, header: Option<&Header>) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    use crate::io::writer::index::header::write_header as write_tabix_header;

    let mut aux = Vec::new();

    if let Some(hdr) = header {
        write_tabix_header(&mut aux, hdr)?;
    }

    let l_aux =
        i32::try_from(aux.len()).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    writer.write_i32_le(l_aux).await?;

    writer.write_all(&aux).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use bstr::BString;

    use super::*;

    #[tokio::test]
    async fn test_write_aux() -> io::Result<()> {
        let mut buf = Vec::new();
        write_aux(&mut buf, None).await?;
        let expected = [0x00, 0x00, 0x00, 0x00];
        assert_eq!(buf, expected);

        let names = [BString::from("sq0"), BString::from("sq1")]
            .into_iter()
            .collect();
        let header = crate::binning_index::index::header::Builder::vcf()
            .set_reference_sequence_names(names)
            .build();

        buf.clear();
        write_aux(&mut buf, Some(&header)).await?;

        let expected = [
            0x24, 0x00, 0x00, 0x00, // l_aux = 36
            0x02, 0x00, 0x00, 0x00, // format = 2 (VCF)
            0x01, 0x00, 0x00, 0x00, // col_seq = 1 (1-based)
            0x02, 0x00, 0x00, 0x00, // col_beg = 2 (1-based)
            0x00, 0x00, 0x00, 0x00, // col_end = None (1-based)
            0x23, 0x00, 0x00, 0x00, // meta = '#'
            0x00, 0x00, 0x00, 0x00, // skip = 0
            0x08, 0x00, 0x00, 0x00, // l_nm = 8
            b's', b'q', b'0', 0x00, // names[0] = "sq0"
            b's', b'q', b'1', 0x00, // names[1] = "sq1"
        ];

        assert_eq!(buf, expected);

        Ok(())
    }
}
