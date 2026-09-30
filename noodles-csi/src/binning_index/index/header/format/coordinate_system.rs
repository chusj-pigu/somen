//! Tabix index format coordinate system.

/// A tabix index format coordinate system.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordinateSystem {
    /// GFF coordinates: 1-based [start, end]
    Gff,
    /// BED coordinates: 0-based [start, end)
    Bed,
}
