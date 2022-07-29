//! The robust and high-performance serializer and deserializer trait.
//!
//! This trait is aimed to provide a simple way to serialize and deserialize
//! wmjtyd's binary data structures, without worrying about the length and
//! some details.
//!
//! For example, see `examples/concept_struct.rs` and our implementations.

use std::io::{self, Read, Write};

/// The serializer for fields.
pub trait FieldSerializer<const LEN: usize>
where
    Self: Sized,
{
    type Err;

    /// Serialize the data as a byte array.
    fn serialize(&self) -> Result<[u8; LEN], Self::Err>;

    /// Serialize the input and write the whole serialized
    /// content to the writer.
    ///
    /// Note that we wrapped the response from `serialize()`
    /// with a `io::Result`, to respect any possible Err.
    fn serialize_to_writer(&self, writer: &mut impl Write) -> Result<io::Result<()>, Self::Err> {
        let serialized = self.serialize()?;

        Ok(writer.write_all(&serialized))
    }
}

/// The deserializer for fields.
pub trait FieldDeserializer<const LEN: usize>
where
    Self: Sized,
{
    type Err;

    /// Desrialize the given data to this type.
    fn deserialize(src: &[u8; LEN]) -> Result<Self, Self::Err>;

    /// Read from the writer, and deserialize it.
    ///
    /// Note that we wrapped the response from `deserialize()`
    /// with a `io::Result`, to respect any possible Err.
    fn deserialize_from_reader(reader: &mut impl Read) -> io::Result<Result<Self, Self::Err>> {
        let mut buf = [0; LEN];
        reader.read_exact(&mut buf)?;

        Ok(Self::deserialize(&buf))
    }
}

/// The serializer for structures.
pub trait StructSerializer
where
    Self: Sized,
{
    type Err;

    fn serialize(&self, writer: &mut impl Write) -> Result<(), Self::Err>;
}

/// The deserializer for structures.
pub trait StructDeserializer
where
    Self: Sized,
{
    type Err;

    fn deserialize(reader: &mut impl Read) -> Result<Self, Self::Err>;
}

macro_rules! serialize_block_builder {
    ($($field:expr),+ => $writer:expr) => {{
        $(
            $field.serialize_to_writer($writer)??
        );+
    }}
}

macro_rules! deserialize_block_builder {
    ($reader:expr => $($field:ident),+) => {{
        Ok(Self {
            $(
                $field: $crate::data::serializer::FieldDeserializer::deserialize_from_reader($reader)??
            ),+
        })
    }}
}

pub(crate) use deserialize_block_builder;
pub(crate) use serialize_block_builder;
