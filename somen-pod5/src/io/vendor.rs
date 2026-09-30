use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[cfg(any(feature = "svb-auto", feature = "svb-neon", feature = "svb-avx2", feature = "svb-ssse3"))]
mod internal {
    use svb::decode_vbz_fused_into;
    use svb::encode_vbz;
    use svb::DecodeError;

    pub fn decode_into(data: &[u8], n: usize, out: &mut [i16]) -> Result<(), DecodeError> {
        decode_vbz_fused_into(data, n, out)
    }

    pub fn encode(samples: &[i16]) -> Vec<u8> {
        encode_vbz(samples)
    }
}
#[cfg(not(any(feature = "svb-auto", feature = "svb-neon", feature = "svb-avx2", feature = "svb-ssse3")))]
mod vbz_fused;
#[cfg(not(any(feature = "svb-auto", feature = "svb-neon", feature = "svb-avx2", feature = "svb-ssse3")))]
mod internal {
    pub use super::vbz_fused::*;
}

pub fn decode(data: &[u8]) -> Result<Vec<i16>, VendorError> {
    todo!()
}

pub fn encode(data: &[i16]) -> Vec<u8> {
    todo!()
}

pub type VendorError = TempError;

//temporary setup
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TempError;

impl Display for TempError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "temporary error used to setup code, to be discarded once no longer needed")
    }
}

impl Error for TempError {}