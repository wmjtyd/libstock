use std::io::{Write, Read, self};

trait ByteSerializer<const LEN: usize>
where
    Self: Sized
{
    type Err;

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

trait ByteDeserializer<const LEN: usize>
where
    Self: Sized
{
    type Err;

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

trait StructSerializer
where
    Self: Sized
{
    type Err;

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Self::Err>;
}

trait StructDeserializer
where
    Self: Sized
{
    type Err;

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Self::Err>;
}

struct Test {
    field1: BoolRepr,
    field2: BoolRepr,
}

struct BoolRepr(bool);

impl ByteSerializer<1> for BoolRepr
where
    Self: Sized
{
    type Err = anyhow::Error;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        let mut dst = [0u8; 1];
        dst[0] = if self.0 { b'1' } else { b'0' };
        Ok(dst)
    }
}

impl ByteDeserializer<1> for BoolRepr
where
    Self: Sized
{
    type Err = anyhow::Error;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let mut dst = BoolRepr(false);
        dst.0 = src[0] != 0;
        Ok(dst)
    }
}

impl StructSerializer for Test
where
    Self: Sized
{
    type Err = anyhow::Error;

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Self::Err> {
        self.field1.serialize_to_writer(writer)??;
        self.field2.serialize_to_writer(writer)??;

        Ok(())
    }
}

impl StructDeserializer for Test
where
    Self: Sized
{
    type Err = anyhow::Error;

    fn deserialize<R: Read>(src: &mut R) -> Result<Self, Self::Err> {
        Ok(Self {
            field1: BoolRepr::deserialize_from_reader(src)??,
            field2: BoolRepr::deserialize_from_reader(src)??,
        })
    }
}

fn main() {
    let t = Test {
        field2: BoolRepr(true),
        field1: BoolRepr(false),
    };
    let stdout = io::stdout();
    let mut lock = stdout.lock();

    t.serialize(&mut lock).unwrap();

    lock.flush().unwrap();
}
