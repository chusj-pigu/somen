/// The signal data associated with a record.
///
/// Signal data may either be stored directly as raw samples or in VBZ-compressed
/// form. This enum abstracts over both representations.
pub enum LargeData<'a> {
    /// Raw, uncompressed signal samples.
    Raw(&'a [i16]),

    /// VBZ-compressed signal samples.
    VBZ(&'a [u8]),
}

impl LargeData<'_> {
    /// Returns `true` if the `LargeData` is stored in it's compressed form.
    pub fn is_compressed(&self) -> bool {
        match self {
            Self::Raw(_) => false,
            Self::VBZ(_) => true,
        }
    }
}