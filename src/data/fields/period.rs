use super::{bimap::create_bimap, Either, FieldDeserializer, FieldError, FieldSerializer};

/// The period of a message (1 byte).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeriodField(pub String);

impl FieldSerializer<1> for PeriodField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        let bit = *PERIOD
            .get_by_left(self.0.as_str())
            .ok_or_else(|| FieldError::UnimplementedPeriod(Either::Left(self.0.to_string())))?;

        Ok([bit])
    }
}

impl FieldDeserializer<1> for PeriodField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0] as u8;

        let name = PERIOD
            .get_by_right(&bit)
            .ok_or(FieldError::UnimplementedPeriod(Either::Right(bit)))?;

        Ok(Self(name.to_string()))
    }
}

create_bimap!(PERIOD {
    &'static str => u8,
    "1m" => 1,
    "5m" => 2,
    "30m" => 3,
    "1h" => 4,
});
