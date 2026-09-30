// This file contains code derived from `svb` by James Ferguson.
// Original project: Psy-Fer/svb
// svb 0.3.0, commit a2fad80
// Licensed under the MIT License;
//
// MIT License
//
// Copyright (c) 2026 James Ferguson
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//!

use std::error::Error;
use std::fmt::Display;

/// Errors that can occur when decoding a StreamVByte-encoded byte slice.
///
/// An empty input with a nonzero sample count produces
/// [`DecodeError::ControlStreamTooShort`].
#[derive(Debug)]
pub enum DecodeError {
    /// The data stream ended before all `n` values could be decoded.
    ///
    /// `index` is the zero-based index of the first value whose bytes were
    /// missing. This usually means `n` was larger than the number of values
    /// that were actually encoded.
    DataTruncated { index: usize },
    /// The control (tag) stream is shorter than required for `n` values.
    ///
    /// `need` is the number of control bytes required; `have` is how many
    /// were present in `data`.
    ControlStreamTooShort { need: usize, have: usize },
    /// The frame's version byte is not one this crate knows how to decode.
    ///
    /// Wire formats that embed a version byte (e.g. ex-zd) use this to
    /// signal forward-incompatible changes rather than silently
    /// misinterpreting the payload.
    UnsupportedVersion { version: u8 },
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::DataTruncated { index } => write!(f, "data truncated: expected more bytes at value {index}"),
            DecodeError::ControlStreamTooShort { need, have } => write!(f, "control stream shorter than expected: need {need} bytes, have {have}"),
            DecodeError::UnsupportedVersion { version } => write!(f, "unsupported format version: {version}"),
        }
    }
}

impl Error for DecodeError {}

/// Decodes vbz into [i16]
pub fn decode_into(
    data: &[u8],
    n: usize,
    out: &mut Vec<i16>,
) -> Result<(), DecodeError> {
    if n == 0 {
        return Ok(());
    }
    let ctrl_len = n.div_ceil(8);
    if data.len() < ctrl_len {
        return Err(DecodeError::ControlStreamTooShort {
            need: ctrl_len,
            have: data.len(),
        });
    }
    let ctrl = &data[..ctrl_len];
    let data_bytes = &data[ctrl_len..];

    let mut acc = 0i16;
    let mut data_pos = 0usize;
    for i in 0..n {
        let bit = (ctrl[i / 8] >> (i % 8)) & 1;
        let raw = if bit == 0 {
            if data_pos >= data_bytes.len() {
                return Err(DecodeError::DataTruncated { index: i });
            }
            let v = data_bytes[data_pos] as u16;
            data_pos += 1;
            v
        } else {
            if data_pos + 2 > data_bytes.len() {
                return Err(DecodeError::DataTruncated { index: i });
            }
            let v = u16::from_le_bytes([data_bytes[data_pos], data_bytes[data_pos + 1]]);
            data_pos += 2;
            v
        };
        // zigzag decode: (raw >> 1) ^ -(raw & 1)  [wrapping u16 negate]
        let delta = ((raw >> 1) ^ (0u16.wrapping_sub(raw & 1))) as i16;
        acc = acc.wrapping_add(delta);
        out.push(acc);
    }
    Ok(())
}


/// Encodes [i16] into vbz
pub fn encode(
    samples: &[i16]
) -> Vec<u8> {
    if samples.is_empty() {
        return Vec::new();
    }

    let ctrl_len = samples.len().div_ceil(8);

    // Worst case: control bytes + 2 bytes per sample.
    let mut out = Vec::with_capacity(ctrl_len + samples.len() * 2);

    // Reserve space for the control stream. We'll fill it in afterward.
    out.resize(ctrl_len, 0);

    let mut prev = 0i16;

    for (i, &sample) in samples.iter().enumerate() {
        // VBZ stores the first difference from zero, then successive
        // differences between samples.
        let delta = sample.wrapping_sub(prev);
        prev = sample;

        // Zigzag encode i16 -> u16.
        //
        // Positive values become 0, 2, 4, ...
        // Negative values become 1, 3, 5, ...
        let raw = ((delta as u16) << 1) ^ ((delta >> 15) as u16);

        if raw <= u8::MAX as u16 {
            // One-byte encoding.
            out.push(raw as u8);
        } else {
            // Two-byte encoding.
            // Control bit 1 means two bytes.
            out[i / 8] |= 1u8 << (i % 8);
            out.extend_from_slice(&raw.to_le_bytes());
        }
    }

    out
}

#[test]
fn encode_decode_delta_boundaries() {
    let samples = [
        0i16,
        127,
        0,
        -128,
        0,
        128,
        0,
        -129,
        0,
        i16::MAX,
        i16::MIN,
    ];

    let encoded = encode(&samples);

    let mut decoded = Vec::with_capacity(samples.len());
    decode_into(&encoded, samples.len(), &mut decoded).unwrap();

    assert_eq!(decoded, samples);
}

#[test]
fn encode_decode_control_bytes_and_truncation() {
    let samples = [0, 127, 255, -1, 0, 1, 2, 3, 4];
    let encoded = encode(&samples);

    // The third and fourth deltas take two bytes, and the ninth starts a new control byte.
    assert_eq!(&encoded[..2], &[0b0000_1100, 0]);

    let mut decoded = Vec::new();
    decode_into(&encoded, samples.len(), &mut decoded).unwrap();
    assert_eq!(decoded, samples);

    assert!(matches!(
        decode_into(&encoded[..1], samples.len(), &mut Vec::new()),
        Err(DecodeError::ControlStreamTooShort { need: 2, have: 1 })
    ));
    assert!(matches!(
        decode_into(&encoded[..encoded.len() - 1], samples.len(), &mut Vec::new()),
        Err(DecodeError::DataTruncated { index: 8 })
    ));
}

#[test]
fn decode_known_vbz_bytes() {
    // Four one-byte zigzag deltas: 0, -1, 2, and 127.
    let mut samples = Vec::new();
    decode_into(&[0, 0, 1, 4, 254], 4, &mut samples).unwrap();
    assert_eq!(samples, [0, -1, 1, 128]);
    assert_eq!(encode(&samples), [0, 0, 1, 4, 254]);

    // A two-byte zigzag delta of 128.
    samples.clear();
    decode_into(&[1, 0, 1], 1, &mut samples).unwrap();
    assert_eq!(samples, [128]);
    assert_eq!(encode(&samples), [1, 0, 1]);
}
