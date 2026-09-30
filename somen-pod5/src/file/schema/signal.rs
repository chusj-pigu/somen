// standard

// third party

// local
use crate::{
    file::schema::ColumnSchema,
    record::batch::internal::arrays::FieldType
};

/// todo
#[derive(Debug, Copy, Clone)]
pub enum SignalSchema {
    /// Globally-unique identifier for the read the data came from.
    /// This aids recovery and consistency checking.
    ReadId,

    /// The actual signal.
    /// The encoding of the data must the same for all reads in the file,
    /// and is determined by the choice of logical type.
    /// `LargeList(Int16)` is the uncompressed storage option.
    /// Readers that do not recognize the logical type of this column
    /// will be unable to decode the signal data.
    Signal,

    /// The number of samples stored in this row.
    /// Allows skipping over compressed chunks easily,
    /// also necessary for decoding StreamVByte-encoded data.
    Samples,
}

impl ColumnSchema for SignalSchema {
    const COLUMNS: &'static [Self] = &[
        SignalSchema::ReadId,
        SignalSchema::Signal,
        SignalSchema::Samples,
    ];

    fn get_type(&self) -> FieldType {
        match self {
            SignalSchema::ReadId => FieldType::Uuid,
            SignalSchema::Signal => FieldType::LargeData,
            SignalSchema::Samples => FieldType::UInt32,
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            SignalSchema::ReadId => "read_id",
            SignalSchema::Signal => "signal",
            SignalSchema::Samples => "samples",
        }
    }
}