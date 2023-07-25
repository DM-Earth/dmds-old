use std::io::{self, prelude::*};

/// Describes a type that can be encoded and decoded as bytes.
pub trait Data: Sized {
    /// Encode into a writer.
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;

    /// Decode from a reader.
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self>;

    fn bytes_len(&self) -> usize;
}

impl Data for u8 {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&mut [*self])
    }

    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = [0; 1];
        reader.read_exact(&mut value)?;
        Ok(value[0])
    }

    fn bytes_len(&self) -> usize {
        1
    }
}
