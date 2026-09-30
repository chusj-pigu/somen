use noodles_bgzf as bgzf;
use noodles_core::Position;

use super::Index;
use crate::binning_index::index::reference_sequence::bin::Chunk;

/// A linear index.
pub type LinearIndex = Vec<bgzf::VirtualPosition>;

impl Index for LinearIndex {
    fn min_offset(&self, min_shift: u8, _: u8, start: Position) -> bgzf::VirtualPosition {
        let window_size = 1 << min_shift;
        let i = (usize::from(start) - 1) / window_size;
        self.get(i).copied().unwrap_or_default()
    }

    fn last_first_start_position(&self) -> Option<bgzf::VirtualPosition> {
        self.last().copied()
    }

    fn update(&mut self, min_shift: u8, _: u8, _: Position, end: Position, chunk: Chunk) {
        let window_size = 1 << min_shift;
        let end_index = (usize::from(end) - 1) / window_size;
        let new_len = end_index + 1;

        if new_len > self.len() {
            self.resize(new_len, chunk.start());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update() -> Result<(), noodles_core::position::TryFromIntError> {
        const MIN_SHIFT: u8 = 14;
        const DEPTH: u8 = 5;

        let mut index = LinearIndex::new();

        let start = Position::try_from(16385)?;
        let end = Position::try_from(65536)?;
        let chunk = Chunk::new(
            bgzf::VirtualPosition::from(8),
            bgzf::VirtualPosition::from(13),
        );
        index.update(MIN_SHIFT, DEPTH, start, end, chunk);

        assert_eq!(
            index,
            [
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
            ]
        );

        let start = Position::try_from(32769)?;
        let end = Position::try_from(49152)?;
        let chunk = Chunk::new(
            bgzf::VirtualPosition::from(13),
            bgzf::VirtualPosition::from(21),
        );
        index.update(MIN_SHIFT, DEPTH, start, end, chunk);

        assert_eq!(
            index,
            [
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
            ]
        );

        let start = Position::try_from(98305)?;
        let end = Position::try_from(114688)?;
        let chunk = Chunk::new(
            bgzf::VirtualPosition::from(21),
            bgzf::VirtualPosition::from(34),
        );
        index.update(MIN_SHIFT, DEPTH, start, end, chunk);

        assert_eq!(
            index,
            [
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(8),
                bgzf::VirtualPosition::from(21),
                bgzf::VirtualPosition::from(21),
                bgzf::VirtualPosition::from(21),
            ]
        );

        Ok(())
    }
}
