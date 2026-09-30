// standard
use std::fmt;
// third party

// local


/// A zero-copy, read-only view of a 128-bit UUID.
///
/// This type provides a strongly typed wrapper around a borrowed
/// 16-byte UUID stored within record data.
/// It intentionally exposes only the UUID's representation, not
/// higher-level operations such as parsing or version inspection.
///
/// Applications requiring UUID-specific operations,
/// such as version inspection or parsing,
/// can construct a `uuid::Uuid` from the underlying bytes.
#[repr(transparent)]
#[must_use]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub struct Uuid<'a>(&'a [u8; 16]);

impl<'a> Uuid<'a> {
    /// Creates a borrowed view over `bytes`.
    ///
    /// The returned reference points directly to the provided array.
    /// No allocation or copy is performed.
    #[inline]
    pub const fn from_bytes(bytes: &'a [u8; 16]) -> Self {
        Uuid(bytes)
    }

    /// Returns a reference to the underlying UUID bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Returns a copy of the underlying UUID bytes.
    #[inline]
    #[must_use]
    pub const fn to_bytes(&self) -> [u8; 16] {
        *self.0
    }
}

impl<'a> fmt::Display for Uuid<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        const DASH: u8 = b'-';

        let b = self.0;

        // SAFETY: All bytes are valid UTF-8 via ASCII-based definition(0-9, a-f, '-').
        unsafe {
            f.write_str(
                std::str::from_utf8_unchecked(&[
                    HEX[(b[0] >> 4) as usize], HEX[(b[0] & 0x0F) as usize],
                    HEX[(b[1] >> 4) as usize], HEX[(b[1] & 0x0F) as usize],
                    HEX[(b[2] >> 4) as usize], HEX[(b[2] & 0x0F) as usize],
                    HEX[(b[3] >> 4) as usize], HEX[(b[3] & 0x0F) as usize],
                    DASH,
                    HEX[(b[4] >> 4) as usize], HEX[(b[4] & 0x0F) as usize],
                    HEX[(b[5] >> 4) as usize], HEX[(b[5] & 0x0F) as usize],
                    DASH,
                    HEX[(b[6] >> 4) as usize], HEX[(b[6] & 0x0F) as usize],
                    HEX[(b[7] >> 4) as usize], HEX[(b[7] & 0x0F) as usize],
                    DASH,
                    HEX[(b[8] >> 4) as usize], HEX[(b[8] & 0x0F) as usize],
                    HEX[(b[9] >> 4) as usize], HEX[(b[9] & 0x0F) as usize],
                    DASH,
                    HEX[(b[10] >> 4) as usize], HEX[(b[10] & 0x0F) as usize],
                    HEX[(b[11] >> 4) as usize], HEX[(b[11] & 0x0F) as usize],
                    HEX[(b[12] >> 4) as usize], HEX[(b[12] & 0x0F) as usize],
                    HEX[(b[13] >> 4) as usize], HEX[(b[13] & 0x0F) as usize],
                    HEX[(b[14] >> 4) as usize], HEX[(b[14] & 0x0F) as usize],
                    HEX[(b[15] >> 4) as usize], HEX[(b[15] & 0x0F) as usize],
                ])
            )
        }
    }
}

impl<'a> PartialEq<[u8; 16]> for Uuid<'a> {
    #[inline]
    fn eq(&self, other: &[u8; 16]) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<[u8]> for Uuid<'a> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.0 == other
    }
}

impl<'a> AsRef<[u8; 16]> for Uuid<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8; 16] {
        self.0
    }
}

impl<'a> AsRef<[u8]> for Uuid<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}