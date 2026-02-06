use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};

use super::rgb::RgbColor;

impl Serialize for RgbColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("#{:0>6X}", self.code()))
        } else {
            serializer.serialize_u32(self.code())
        }
    }
}

impl<'de> Deserialize<'de> for RgbColor {
    #[allow(
        clippy::cast_sign_loss,
        clippy::manual_range_contains,
        clippy::cast_possible_truncation
    )]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        macro_rules! impl_visit_unsigned_small {
            ($t:ty, $i:ident) => {
                fn $i<E: de::Error>(self, v: $t) -> Result<Self::Value, E> {
                    Ok(RgbColor::hex(v.into()))
                }
            };
        }
        macro_rules! impl_visit_signed_small {
            ($t:ty, $i:ident) => {
                fn $i<E: de::Error>(self, v: $t) -> Result<Self::Value, E> {
                    if v < 0 {
                        return Err(E::invalid_value(Unexpected::Signed(v.into()), &self));
                    }
                    Ok(RgbColor::hex(v as u32))
                }
            };
        }
        macro_rules! impl_visit_unsigned_big {
            ($t:ty, $i:ident) => {
                fn $i<E: de::Error>(self, v: $t) -> Result<Self::Value, E> {
                    if v > 0xFFFFFF {
                        return Err(E::invalid_value(Unexpected::Unsigned(v.into()), &self));
                    }
                    Ok(RgbColor::hex(v as u32))
                }
            };
        }
        macro_rules! impl_visit_signed_big {
            ($t:ty, $i:ident) => {
                fn $i<E: de::Error>(self, v: $t) -> Result<Self::Value, E> {
                    if v < 0 || v > 0xFFFFFF {
                        return Err(E::invalid_value(Unexpected::Signed(v.into()), &self));
                    }
                    Ok(RgbColor::hex(v as u32))
                }
            };
        }
        macro_rules! impl_visit_128 {
            ($t:ty, $i:ident) => {
                fn $i<E: de::Error>(self, v: $t) -> Result<Self::Value, E> {
                    #[allow(unused_comparisons)]
                    if v < 0 || v > 0xFFFFFF {
                        return Err(E::invalid_value(Unexpected::Other(stringify!($t)), &self));
                    }
                    Ok(RgbColor::hex(v as u32))
                }
            };
        }

        struct RgbColorVisitor;

        impl Visitor<'_> for RgbColorVisitor {
            type Value = RgbColor;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex code, name, or integer between 0x000000 and 0xFFFFFF")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                RgbColor::named(v).ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
            }

            impl_visit_signed_small!(i8, visit_i8);
            impl_visit_signed_small!(i16, visit_i16);
            impl_visit_signed_big!(i32, visit_i32);
            impl_visit_signed_big!(i64, visit_i64);
            impl_visit_unsigned_small!(u8, visit_u8);
            impl_visit_unsigned_small!(u16, visit_u16);
            impl_visit_unsigned_big!(u32, visit_u32);
            impl_visit_unsigned_big!(u64, visit_u64);
            impl_visit_128!(i128, visit_i128);
            impl_visit_128!(u128, visit_u128);
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(RgbColorVisitor)
        } else {
            deserializer.deserialize_u32(RgbColorVisitor)
        }
    }
}
