use std::{
    io::{self, prelude::*},
    ops::Add,
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

pub trait RangedSeek: Ord {
    fn min() -> Self;
    fn max() -> Self;
}

pub struct RangeTable<T>
where
    T: RangedSeek + Add,
{
    separation: T,
    chunks: usize,
}
