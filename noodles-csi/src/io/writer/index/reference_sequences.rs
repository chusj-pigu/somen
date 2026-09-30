mod bins;
mod metadata;

use std::io::{self, Write};

use self::{bins::write_bins, metadata::write_metadata};
use crate::{
    binning_index::{
        ReferenceSequence as _,
        index::{ReferenceSequence, reference_sequence::index::BinnedIndex},
    },
    io::writer::num::write_i32_le,
};

pub(super) fn write_reference_sequences<W>(
    writer: &mut W,
    depth: u8,
    reference_sequences: &[ReferenceSequence<BinnedIndex>],
) -> io::Result<()>
where
    W: Write,
{
    write_reference_sequence_count(writer, reference_sequences.len())?;

    for reference_sequence in reference_sequences {
        write_reference_sequence(writer, depth, reference_sequence)?;
    }

    Ok(())
}

fn write_reference_sequence_count<W>(
    writer: &mut W,
    reference_sequence_count: usize,
) -> io::Result<()>
where
    W: Write,
{
    let n = i32::try_from(reference_sequence_count)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    write_i32_le(writer, n)
}

fn write_reference_sequence<W>(
    writer: &mut W,
    depth: u8,
    reference_sequence: &ReferenceSequence<BinnedIndex>,
) -> io::Result<()>
where
    W: Write,
{
    write_bins(
        writer,
        depth,
        reference_sequence.bins(),
        reference_sequence.index(),
        reference_sequence.metadata(),
    )
}
