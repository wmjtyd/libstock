//! The internal methods for constructing methods for compatibility.

macro_rules! compat_enc {
    (
        enc = $enc_method:ident,
        dec = $dec_method:ident,
        crawl = $crawler_type:ty,
        result = $result_type:tt,
        structure = $structure_type:ty
    ) => {
        #[cfg(feature = "compat-v0_3")]
        #[deprecated = "Better using `BboStructure` directly for better performance. See CHANGELOG for details."]
        #[doc = concat!("Encode a [`", stringify!($crawler_type), "`] to bytes.")]
        pub fn $enc_method(src: &$crawler_type) -> $result_type<Vec<u8>> {
            let mut buf = Vec::new();
            let structure: $structure_type = src.try_into()?;

            structure.serialize(&mut buf)?;
            Ok(buf)
        }

        #[cfg(feature = "compat-v0_3")]
        #[deprecated = "Better using `BboStructure` directly for better performance. See CHANGELOG for details."]
        #[doc = concat!("Decode the specified bytes to a [`", stringify!($crawler_type), "`].")]
        pub fn $dec_method(mut payload: &[u8]) -> $result_type<$crawler_type> {
            let structure: $structure_type = $crate::data::serializer::StructDeserializer::deserialize(&mut payload)?;

            structure.try_into()
        }
    }
}

pub(crate) use compat_enc;
