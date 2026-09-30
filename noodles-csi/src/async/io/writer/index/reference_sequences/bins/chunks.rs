use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use crate::binning_index::index::reference_sequence::bin::Chunk;

pub(super) async fn write_chunks<W>(writer: &mut W, chunks: &[Chunk]) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    write_chunk_count(writer, chunks.len()).await?;

    for chunk in chunks {
        write_chunk(writer, chunk).await?;
    }

    Ok(())
}

async fn write_chunk_count<W>(writer: &mut W, chunk_count: usize) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let n =
        i32::try_from(chunk_count).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    writer.write_i32_le(n).await
}

async fn write_chunk<W>(writer: &mut W, chunk: &Chunk) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let chunk_beg = u64::from(chunk.start());
    writer.write_u64_le(chunk_beg).await?;

    let chunk_end = u64::from(chunk.end());
    writer.write_u64_le(chunk_end).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use noodles_bgzf as bgzf;

    use super::*;

    #[tokio::test]
    async fn test_write_chunks() -> io::Result<()> {
        let mut buf = Vec::new();
        let chunk = Chunk::new(
            bgzf::VirtualPosition::from(8),
            bgzf::VirtualPosition::from(13),
        );
        write_chunks(&mut buf, &[chunk]).await?;

        let expected = [
            0x01, 0x00, 0x00, 0x00, // n_chunk = 1
            0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // chunk_beg[0] = 8
            0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // chunk_end[0] = 13
        ];

        assert_eq!(buf, expected);

        Ok(())
    }
}
