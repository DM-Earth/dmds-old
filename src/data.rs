use std::{
    io::{self, prelude::*},
    ops::Sub,
};

/// Describes a type that can be encoded bytes.
pub trait Encode {
    /// Encode into a writer.
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;

    /// The length of this instance, in bytes.
    fn bytes_len(&self) -> usize;
}

/// Describes a type that can be decoded from bytes.
pub trait Decode: Sized {
    /// Decode from a reader.
    fn decode<R: Read>(reader: R) -> io::Result<Self>;
}

pub trait Data: Encode + Decode {
    type Identifier: Encode + Decode;
}

pub trait RangedSeek: Sub<Output = Self::Raw> + Sized + Into<Self::Raw> {
    type Raw: RangedSeek;

    const RS_MIN: Self;
    const RS_MAX: Self;

    fn rs_div_floor(self, rhs: Self::Raw) -> usize;

    fn rs_div_usize_ceil(self, rhs: usize) -> Self::Raw;
}

impl RangedSeek for u64 {
    type Raw = Self;

    const RS_MIN: Self = u64::MIN;

    const RS_MAX: Self = u64::MAX;

    fn rs_div_floor(self, rhs: Self::Raw) -> usize {
        (self / rhs) as usize
    }

    fn rs_div_usize_ceil(self, rhs: usize) -> Self::Raw {
        self / rhs as u64 + if self % rhs as u64 == 0 { 0 } else { 1 }
    }
}

pub struct RangeTable<T>
where
    T: RangedSeek,
{
    separation: T,
    chunk_count: usize,
}

impl<T: RangedSeek<Raw = T>> RangeTable<T> {
    pub fn from_count<U: RangedSeek<Raw = T>>(chunk_count: usize) -> Self {
        Self {
            separation: (U::RS_MAX - U::RS_MIN).rs_div_usize_ceil(chunk_count),
            chunk_count,
        }
    }

    pub fn from_separation<U: RangedSeek<Raw = T> + Copy>(separation: U) -> Self {
        Self {
            separation: separation.into(),
            chunk_count: (U::RS_MAX - U::RS_MIN).rs_div_floor(separation.into()),
        }
    }
}

impl<T: RangedSeek + Copy> RangeTable<T> {
    pub fn get(&self, id: T) -> Option<usize> {
        let raw = id.rs_div_floor(self.separation.into());

        if raw <= self.chunk_count {
            Some(raw)
        } else {
            None
        }
    }
}
