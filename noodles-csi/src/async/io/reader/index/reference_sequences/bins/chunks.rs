use noodles_bgzf as bgzf;
use tokio::io::{self, AsyncRead, AsyncReadExt};

use crate::binning_index::index::reference_sequence::bin::Chunk;

pub(super) async fn read_chunks<R>(reader: &mut R) -> io::Result<Vec<Chunk>>
where
    R: AsyncRead + Unpin,
{
    let chunk_count = read_chunk_count(reader).await?;

    let mut chunks = Vec::with_capacity(chunk_count);

    for _ in 0..chunk_count {
        let chunk = read_chunk(reader).await?;
        chunks.push(chunk);
    }

    Ok(chunks)
}

async fn read_chunk_count<R>(reader: &mut R) -> io::Result<usize>
where
    R: AsyncRead + Unpin,
{
    reader
        .read_i32_le()
        .await
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
}

async fn read_chunk<R>(reader: &mut R) -> io::Result<Chunk>
where
    R: AsyncRead + Unpin,
{
    let chunk_beg = reader
        .read_u64_le()
        .await
        .map(bgzf::VirtualPosition::from)?;

    let chunk_end = reader
        .read_u64_le()
        .await
        .map(bgzf::VirtualPosition::from)?;

    Ok(Chunk::new(chunk_beg, chunk_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_chunk_count() -> io::Result<()> {
        assert_eq!(
            read_chunk_count(&mut &[0x00, 0x00, 0x00, 0x00][..]).await?,
            0
        );
        assert_eq!(
            read_chunk_count(&mut &[0x01, 0x00, 0x00, 0x00][..]).await?,
            1
        );
        assert_eq!(
            read_chunk_count(&mut &[0x00, 0x01, 0x00, 0x00][..]).await?,
            256
        );

        assert!(matches!(
            read_chunk_count(&mut io::empty()).await,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));
        assert!(matches!(
            read_chunk_count(&mut &[0xff, 0xff, 0xff, 0xff][..]).await,
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));

        Ok(())
    }

    #[tokio::test]
    async fn test_read_chunk() -> io::Result<()> {
        let src = [
            0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // chunk_beg = 8
            0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // chunk_end = 13
        ];

        let actual = read_chunk(&mut &src[..]).await?;
        let expected = Chunk::new(
            bgzf::VirtualPosition::from(8),
            bgzf::VirtualPosition::from(13),
        );
        assert_eq!(actual, expected);

        assert!(matches!(
            read_chunk(&mut io::empty()).await,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        Ok(())
    }
}
