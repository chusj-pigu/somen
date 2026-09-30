mod bins;
mod metadata;

use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use self::{bins::write_bins, metadata::write_metadata};
use crate::binning_index::{
    ReferenceSequence as _,
    index::{ReferenceSequence, reference_sequence::index::BinnedIndex},
};

pub(super) async fn write_reference_sequences<W>(
    writer: &mut W,
    depth: u8,
    reference_sequences: &[ReferenceSequence<BinnedIndex>],
) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    write_reference_sequence_count(writer, reference_sequences.len()).await?;

    for reference_sequence in reference_sequences {
        write_reference_sequence(writer, depth, reference_sequence).await?;
    }

    Ok(())
}

async fn write_reference_sequence_count<W>(
    writer: &mut W,
    reference_sequence_count: usize,
) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let n = i32::try_from(reference_sequence_count)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    writer.write_i32_le(n).await
}

async fn write_reference_sequence<W>(
    writer: &mut W,
    depth: u8,
    reference_sequence: &ReferenceSequence<BinnedIndex>,
) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    write_bins(
        writer,
        depth,
        reference_sequence.bins(),
        reference_sequence.index(),
        reference_sequence.metadata(),
    )
    .await
}
