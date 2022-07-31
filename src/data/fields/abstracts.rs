use super::{FieldDeserializer, FieldSerializer};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

/// A trait that abstracted a field.
///
/// A field should be:
///
/// - Sized.
/// - Can be serialized to, and deserialized from `[u8; L]`.
pub trait Field<const L: usize>: Sized + FieldSerializer<L> + FieldDeserializer<L> {}

/// If a field is interopable.
/// 
/// It means:
/// 
/// - T –From→ Field(T)
/// - Field(T) –To→ T
/// - Field(T) –Deref→ &T
/// - Field(T) –DerefMut→ &mut T
pub trait Interopable<T>: From<T> + Into<T> + Deref<Target = T> + DerefMut<Target = T> {}

/// This trait also require the field implements
/// `PartialEq`, `Eq`, `Hash` and `Clone`. It is not required
/// but recommended being implemented.
pub(super) trait HighStandardField<T, const L: usize>:
    Field<L> + Interopable<T> + PartialEq + Eq + Hash + Clone
{
}

/// Automatically derive [`Field`] and [`HighStandardField`],
/// without writing any procedural macro.
///
/// Internal use only.
macro_rules! derive_hsf {
    ($target:ty, $inner_type:ty, $len:expr) => {
        impl $crate::data::fields::abstracts::Field<$len> for $target {}
        impl $crate::data::fields::abstracts::HighStandardField<$inner_type, $len> for $target {}
    };
}

/// Automatically derive `From`, `Into`, `Deref`, `DerefMut`
/// for basic field (`Field(T)`).
macro_rules! derive_interop_converters {
    ($target:ty, $inner_type:ty) => {
        impl From<$inner_type> for $target {
            fn from(r: $inner_type) -> Self {
                Self(r)
            }
        }

        impl From<$target> for $inner_type {
            fn from(r: $target) -> Self {
                r.0
            }
        }

        impl std::ops::Deref for $target {
            type Target = $inner_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $target {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl $crate::data::fields::abstracts::Interopable<$inner_type> for $target {}
    };
}

pub(super) use {derive_hsf, derive_interop_converters};
