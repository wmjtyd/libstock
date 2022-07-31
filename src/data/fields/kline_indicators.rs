use typed_builder::TypedBuilder;

use super::{DecimalField, FieldSerializer, FieldError, FieldDeserializer, Field};

// The explanations are picked from
// https://moneymate.space/k線/
// Thanks to Cindy!

/// The indicators for k-line (30 bytes).
/// 
/// It includes `open`, `high`, `low`,
/// `close`, and `volume`.
/// 
/// For more information, see <https://moneymate.space/k線/>.
#[derive(Clone, Debug, PartialEq, Eq, Hash, TypedBuilder)]
pub struct KlineIndicatorsField {
    /// 開盤價 (5 bytes)
    /// 
    /// 開市後，第一筆成交的價格。
    #[builder(setter(into))]
    pub open: DecimalField<5>,
    
    /// 最高價 (5 bytes)
    /// 
    /// 期間內，買賣的最高價。
    #[builder(setter(into))]
    pub high: DecimalField<5>,
    
    /// 最低價 (5 bytes)
    /// 
    /// 期間內，買賣的最低價。
    #[builder(setter(into))]
    pub low: DecimalField<5>,
    
    /// 收盤價 (5 bytes)
    /// 
    /// 閉市前，最後一筆成交的價格。
    #[builder(setter(into))]
    pub close: DecimalField<5>,
    
    /// 交易量 (10 bytes)
    #[builder(setter(into))]
    pub volume: DecimalField<10>,
}

impl FieldSerializer<30> for KlineIndicatorsField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 30], Self::Err> {
        let mut dst = [0u8; 30];

        let mut offset = 0;

        dst[offset..offset + 5].copy_from_slice(&self.open.serialize()?);
        offset += 5;
        dst[offset..offset + 5].copy_from_slice(&self.high.serialize()?);
        offset += 5;
        dst[offset..offset + 5].copy_from_slice(&self.low.serialize()?);
        offset += 5;
        dst[offset..offset + 5].copy_from_slice(&self.close.serialize()?);
        offset += 5;
        dst[offset..offset + 10].copy_from_slice(&self.volume.serialize()?);

        Ok(dst)
    }
}

impl FieldDeserializer<30> for KlineIndicatorsField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 30]) -> Result<Self, Self::Err> {
        let mut offset = 0;

        let open = DecimalField::deserialize(arrayref::array_ref![src, offset, 5])?;
        offset += 5;
        let high = DecimalField::deserialize(arrayref::array_ref![src, offset, 5])?;
        offset += 5;
        let low = DecimalField::deserialize(arrayref::array_ref![src, offset, 5])?;
        offset += 5;
        let close = DecimalField::deserialize(arrayref::array_ref![src, offset, 5])?;
        offset += 5;
        let volume = DecimalField::deserialize(arrayref::array_ref![src, offset, 10])?;
        offset += 10;

        Ok(Self {
            open,
            high,
            low,
            close,
            volume,
        })
    }
}

impl Field<30> for KlineIndicatorsField {}
