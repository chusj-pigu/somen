//! Tabix index header format and coordinate system.

mod coordinate_system;

pub use self::coordinate_system::CoordinateSystem;

/// A tabix index format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// A generic format with a defined coordinate system.
    Generic(CoordinateSystem),
    /// The SAM (Sequence Alignment/Map) format.
    Sam,
    /// The VCF (Variant Call Format) format.
    Vcf,
}

impl Format {
    /// Returns the coordinate system of the format.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::binning_index::index::header::{format::CoordinateSystem, Format};
    ///
    /// let format = Format::Generic(CoordinateSystem::Bed);
    /// assert_eq!(format.coordinate_system(), CoordinateSystem::Bed);
    ///
    /// assert_eq!(Format::Sam.coordinate_system(), CoordinateSystem::Gff);
    /// assert_eq!(Format::Vcf.coordinate_system(), CoordinateSystem::Gff);
    /// ```
    pub fn coordinate_system(&self) -> CoordinateSystem {
        match self {
            Self::Generic(coordinate_system) => *coordinate_system,
            Self::Sam | Self::Vcf => CoordinateSystem::Gff,
        }
    }
}
