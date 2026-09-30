use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use crate::file::IndexOutOfBounds;
use crate::io::decoder::{DecoderError, Order};
use crate::io::reader::{ConcurrencyMode, ReferenceModel};
use crate::record::batch::internal::SignalColumns;
use crate::record::{Record, SignalData, SignalRecord};
use crate::record::internal::contracts::SignalRecordContract;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The index within a single [`Buffer`](SignalBuffer).
pub struct BufferIndex(u64);

impl BufferIndex {
    /// Returns a new `BufferIndex`.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the value of the `BufferIndex`.
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for BufferIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferRange {
    segment_start: BufferIndex,
    segment_length: u32,
}

impl BufferRange {
    fn new(segment_start: BufferIndex, segment_length: u32) -> Result<Self, LengthOverflow> {
        if segment_length > (SignalBufferMax::MAX as u32) {
            return Err(LengthOverflow(segment_length as u64))
        }
        Ok(
            Self {
                segment_start,
                segment_length,
            }
        )
    }

    fn values(&self) -> (u64, u32) {
        (
            self.segment_start.value(),
            self.segment_length,
        )
    }

    fn segment_start(&self) -> BufferIndex {
        self.segment_start
    }
    fn segment_length(&self) -> u32 {
        self.segment_length
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// todo
pub enum BufferError {
    /// The requested row index falls outside the valid bounds of the batch.
    BufferIndexOutOfBounds(IndexOutOfBounds),

    BufferRangeOutOfBounds {
        buffer_range: BufferRange,
        source_max: u64,
    }
}

impl Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BufferIndexOutOfBounds(index) => write!(f, "{}", index),
            Self::BufferRangeOutOfBounds { buffer_range, source_max } => {
                write!(
                    f,
                    "Range (start {}; length {}) out of source range [0-{source_max}]",
                    buffer_range.segment_start().value(),
                    buffer_range.segment_length(),
                )
            }
        }
    }
}

impl Error for BufferError {}

/// todo
#[derive(Debug)]
pub enum ParameterError {
    OverlapTooSmall,
    OverlapTooBig {
        overlap: usize,
        length: usize,
    },
    LengthTooSmall,
    LengthTooBigMax(LengthOverflow),
    LengthTooBigSource {
        length: usize,
        source: usize,
    },
}

impl Display for ParameterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverlapTooSmall => f.write_str("Overlap of size 0"),
            Self::OverlapTooBig { overlap, length } => write!(f,"Overlap {overlap} equal or bigger to Length {length}"),
            Self::LengthTooSmall => f.write_str("Length of size 0 or 1"),
            Self::LengthTooBigMax(_) => write!(f,"Length too big"),
            Self::LengthTooBigSource { length, source } => write!(f,"Length {length} bigger than source array length {source}"),
        }
    }
}

impl Error for ParameterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LengthTooBigMax(error) => Some(error),
            _ => None,
        }
    }
}

/// todo
#[derive(Debug)]
pub struct LengthOverflow(u64);

impl Display for LengthOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Length {} bigger than SignalBufferMax::MAX", self.0)
    }
}

impl Error for LengthOverflow {}

pub trait SignalBufferContract<R: ReferenceModel>{}

pub struct SignalBufferMax;

impl SignalBufferMax {
    /// todo
    pub const MAX: u64 = isize::MAX.unbounded_shr(2) as u64;
}

/// todo
pub struct SignalBuffer<R: ReferenceModel>{
    /// todo
    signal_records: Vec<SignalRecord<R>>,

    /// todo
    record_indexes: Vec<BufferIndex>,

    /// todo
    full_buffer: Option<Vec<i16>>,

    /// todo
    samples: u64,
}

impl<R: ReferenceModel> SignalBuffer<R> {
    /// todo
    pub(crate) unsafe fn try_from_order(order: Order<R::ConcurrencyMode>)
        -> Result<
            SignalBuffer<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<SignalColumns>>,
            DecoderError
        >
    {
        // Safety
        // Caller must ensure the order is ready before passing it in
        let data = unsafe {
            order.to_tasks()
        };
        let mut sum = 0u64;
        let mut task_errors = Vec::with_capacity(data.len());
        let mut signal_records = Vec::with_capacity(data.len());
        let mut record_indexes = Vec::with_capacity(data.len());
        for task in data {
            match task.data() {
                SignalData::Raw | SignalData::DecompressedVBZ(_) => {
                    match task.samples() {
                        Ok(samples) => {
                            if let Err(e) = task.signal() {
                                task_errors.push(e)
                            }
                            if let Err(e) = task.read_id() {
                                task_errors.push(e)
                            }
                            if task_errors.is_empty() {
                                sum = sum.saturating_add(u64::from(samples));
                                let index = BufferIndex::from(sum.saturating_sub(1));
                                record_indexes.push(index);
                                signal_records.push(*task);
                            }
                        },
                        Err(e) => task_errors.push(e)
                    }
                },
                SignalData::Error(e) => task_errors.push(e.clone()),
                SignalData::Unset => panic!("cannot convert an Order which is not ready into a SignalBuffer"),
            }
        }
        if task_errors.is_empty() {
            Ok(SignalBuffer::<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<SignalColumns>>{
                signal_records,
                record_indexes,
                full_buffer: None,
                samples: sum,
            })
        } else {
            Err(
                DecoderError::SignalRecords(
                    Arc::new(task_errors)
                )
            )
        }
    }

    fn samples(&self) -> u64 {
        self.samples
    }

    fn as_signal_records(&self) -> &Vec<SignalRecord<R>> {
         &self.signal_records
    }

    fn to_signal_records(self) -> Vec<SignalRecord<R>> {
        self.signal_records
    }

    fn as_sources(&self) -> Vec<&[i16]> {
        let mut sources = Vec::with_capacity(self.signal_records.len());
        for record in &self.signal_records {
            // Safety;
            // Verified during initialization
            let signal = unsafe {
                record.signal().unwrap_unchecked()
            };
            sources.push(signal)
        }
        sources
    }

    /// todo
    fn fuse(&mut self) -> Result<(), LengthOverflow> {
        if self.full_buffer.is_some() {
            return Ok(())
        }
        if self.samples > SignalBufferMax::MAX {
            return Err(LengthOverflow(self.samples))
        }
        let mut full_buffer: Vec<i16> = Vec::with_capacity(self.samples as usize);
        for record in &self.signal_records {
            // Safety;
            // Verified during initialization
            let signal = unsafe {
                record.signal().unwrap_unchecked()
            };
            full_buffer.extend(signal)
        }
        Ok(())
    }

    fn get_full(&self) -> Option<&Vec<i16>> {
        self.full_buffer.as_ref()
    }

    fn record_index(&self, index: BufferIndex) -> Result<usize, BufferError> {
        if index > *self.record_indexes.last().unwrap() {
            return Err(BufferError::BufferIndexOutOfBounds(
                IndexOutOfBounds(index.value())
            ))
        }
        let record_index = self.record_indexes
            .binary_search(&index)
            .unwrap_or_else(|index| index);
        Ok(record_index)
    }

    fn signal(&self, index: BufferIndex) -> Result<i16, BufferError> {
        let record_index = self.record_index(index)?;
        let local_index = if record_index == 0 {
            index.value() as usize
        } else {
            let sub = self.record_indexes[record_index - 1];
            index.value().strict_sub(sub.value().strict_add(1)) as usize
        };
        let record = &self.signal_records[record_index];
        let signal = record.signal().unwrap()[local_index];
        Ok(signal)
    }

    fn slice(&mut self, buffer_range: BufferRange, mut callback: impl FnMut(&[i16])) -> Result<(), BufferError> {
        let start = buffer_range.segment_start();
        let mut length = buffer_range.segment_length();
        if start.value().strict_add(length as u64) > self.samples {
            return Err(BufferError::BufferRangeOutOfBounds {
                buffer_range,
                source_max: self.samples,
            })
        }
        if let Some(buffer) = &self.full_buffer {
            let start = start.value() as usize;
            let end = start.strict_add(length as usize);
            callback(&buffer[start..end]);
            return Ok(())
        }
        let mut index = self.record_index(start)?;
        let mut record = &self.signal_records[index];
        let start = self.record_indexes[index].value()
            .strict_sub(start.value()) as usize;
        let mut buffer = Vec::with_capacity(length as usize);
        // Safety;
        // Verified during initialization
        let samples = unsafe {
            record.samples().unwrap_unchecked()
        };
        // Safety;
        // Verified during initialization
        let values = unsafe {
            record.signal().unwrap_unchecked()
        };

        if samples >= length {
            callback(&values[start..length as usize]);
            return Ok(())
        }
        // Safety;
        // Just checked above
        unsafe {
            length = length.unchecked_sub(samples);
        }
        buffer.extend_from_slice(&values[start..]);

        loop {
            index += 1;
            record = &self.signal_records[index];

            // Safety;
            // Verified during initialization
            let samples = unsafe {
                record.samples().unwrap_unchecked()
            };
            // Safety;
            // Verified during initialization
            let values = unsafe {
                record.signal().unwrap_unchecked()
            };

            if samples >= length {
                buffer.extend_from_slice(&values[start..length as usize]);
                callback(&buffer);
                return Ok(())
            }
            // Safety;
            // Just checked above
            unsafe {
                length = length.unchecked_sub(samples);
            }
            buffer.extend_from_slice(&values[start..]);
        }
    }


    //recommend call fuze before custom iter, as copying is not cheapened here, or making custom itteration via sources
    fn for_slices_custom(&mut self, buffer_ranges: impl Iterator<Item = BufferRange>, mut callback: impl FnMut(&[i16])) -> Result<(), BufferError> {
        for buffer_range in buffer_ranges {
            self.slice(buffer_range, &mut callback)?;
        }
        Ok(())
    }

    // For direct iteration, this should generally be faster.
    // If you need to do it agian but with different slice parameters however, it should be recomended to go
    // directly with the fuze operation first, as there is a strong chance that different slice parameters will require
    // copying sub slices again.
    fn for_slices(&mut self, length: usize, overlap: usize, mut callback: impl FnMut(&[i16])) -> Result<Option<&[i16]>, ParameterError> {
        if overlap == 0 {
            return Err(ParameterError::OverlapTooSmall)
        }
        if length < 2 {
            return Err(ParameterError::LengthTooSmall)
        }
        if overlap >= length {
            return Err(ParameterError::OverlapTooBig {
                overlap,
                length,
            })
        }
        if length >= SignalBufferMax::MAX as usize {
            return Err(ParameterError::LengthTooBigMax(
                LengthOverflow(length as u64)
            ))
        }
        let mut ceiling = 0u64;
        for record in &self.signal_records {
            // Safety;
            // Verified during initialization
            let samples = unsafe {
                record.samples().unwrap_unchecked() as usize
            };
            ceiling = ceiling.strict_add(samples as u64);
            if length > samples {
                return Err(ParameterError::LengthTooBigSource {
                    length,
                    source: samples,
                })
            }
        }

        // Safety;
        // Validated above
        let step = unsafe {
            length.unchecked_sub(overlap)
        };

        if self.full_buffer.is_some() {
            let iter = (0..ceiling)
                .step_by(step)
                .map(|start_index| {
                    // Safety;
                    // Verified length is valid before
                    unsafe {
                        BufferRange::new(
                            BufferIndex::from(start_index),
                            length as u32,
                        ).unwrap_unchecked()
                    }
                });
            self.for_slices_custom(iter, callback).unwrap();
            let ceiling = ceiling as usize;
            let remain = ceiling.strict_sub(length)
                .strict_rem(step);
            let result = match remain {
                0 => None,
                _ => {
                    // Safety;
                    // just verified above
                    let full_buffer = unsafe {
                        self.full_buffer.as_mut().unwrap_unchecked()
                    };
                    Some(&full_buffer[remain..ceiling])
                },
            };
            return Ok(result)
        }

        let buffer_size = unsafe {
            length.unchecked_sub(1).unchecked_shl(2)
        };

        let mut buffer = Vec::<i16>::with_capacity(buffer_size);

        let mut start = 0;
        let mut end = length;

        let mut record_iter = self.signal_records.iter();
        // Safety;
        // Verified during initialization
        let mut source= unsafe {
            record_iter.next().unwrap().signal().unwrap_unchecked()
        };

        let mut iter = |source: &[i16], start: &mut usize, end: &mut usize| {
            while *end < source.len() {
                callback(&source[*start..*end]);
                *start = start.strict_add(step);
                *end = end.strict_add(step);
            }
        };

        loop {
            iter(source, &mut start, &mut end);
            let source_len = source.len();
            buffer.clear();
            let remaining = &source[start..source.len()];
            source = match record_iter.next() {
                Some(record) => {
                    buffer.extend_from_slice(remaining);
                    // Safety;
                    // Verified during initialization
                    unsafe {
                        record.signal().unwrap_unchecked()
                    }
                },
                None => {
                    return if remaining.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(remaining))
                    }
                }
            };
            end = length.strict_sub(buffer.len());
            while start < source.len() {
                start = start.strict_add(step);
                end = end.strict_add(step);
            }
            buffer.extend_from_slice(&source[0..end.strict_sub(step)]);
            let mut buffer_start = 0;
            let mut buffer_end = length;
            iter(&buffer, &mut buffer_start, &mut buffer_end);
            start = start.strict_sub(source_len);
            end = end.strict_sub(source_len);
        }
    }

    fn iter(&self) -> impl Iterator<Item = &i16> {
        self.signal_records.iter().flat_map(|record| {
            // Safety;
            // Verified during initialization
            unsafe {
                record.signal().unwrap_unchecked().iter()
            }
        })
    }

    /// todo
    fn to_stored(self) -> SignalBuffer<R::StoredReferenceModel<SignalColumns>> {
        let mut signals = Vec::with_capacity(self.signal_records.len());
        for record in self.signal_records {
            signals.push(record.to_stored())
        }
        SignalBuffer {
            signal_records: signals,
            record_indexes: self.record_indexes,
            full_buffer: self.full_buffer,
            samples: self.samples,
        }
    }
}

impl<R: ReferenceModel> SignalBufferContract<R> for SignalBuffer<R> {
    
}