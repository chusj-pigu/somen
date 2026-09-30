mod chunks;

use indexmap::IndexMap;
use noodles_bgzf as bgzf;
use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use self::chunks::write_chunks;
use super::write_metadata;
use crate::binning_index::index::reference_sequence::{Bin, Metadata, index::BinnedIndex};

pub(super) async fn write_bins<W>(
    writer: &mut W,
    depth: u8,
    bins: &IndexMap<usize, Bin>,
    index: &BinnedIndex,
    metadata: Option<&Metadata>,
) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    write_bin_count(writer, metadata.is_some(), bins.len()).await?;

    for (&id, bin) in bins {
        let first_record_start_position = first_record_start_position(index, id);
        write_bin(writer, id, first_record_start_position, bin).await?;
    }

    if let Some(m) = metadata {
        write_metadata(writer, depth, m).await?;
    }

    Ok(())
}

async fn write_bin_count<W>(writer: &mut W, has_metadata: bool, bin_count: usize) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let n = i32::try_from(bin_count)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        .and_then(|n| {
            if has_metadata {
                n.checked_add(1)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "n_bin overflow"))
            } else {
                Ok(n)
            }
        })?;

    writer.write_i32_le(n).await
}

async fn write_bin<W>(
    writer: &mut W,
    id: usize,
    first_record_start_position: bgzf::VirtualPosition,
    bin: &Bin,
) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let bin_id = u32::try_from(id).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    writer.write_u32_le(bin_id).await?;

    let loffset = u64::from(first_record_start_position);
    writer.write_u64_le(loffset).await?;

    write_chunks(writer, bin.chunks()).await?;

    Ok(())
}

fn first_record_start_position(index: &BinnedIndex, id: usize) -> bgzf::VirtualPosition {
    index.get(&id).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_bin_count() -> io::Result<()> {
        let mut buf = Vec::new();

        buf.clear();
        write_bin_count(&mut buf, false, 0).await?;
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);

        buf.clear();
        write_bin_count(&mut buf, true, 0).await?;
        assert_eq!(buf, [0x01, 0x00, 0x00, 0x00]);

        #[cfg(not(target_pointer_width = "16"))]
        {
            buf.clear();
            assert!(matches!(
                write_bin_count(&mut buf, true, usize::MAX).await,
                Err(e) if e.kind() == io::ErrorKind::InvalidInput
            ));
        }

        Ok(())
    }
}
