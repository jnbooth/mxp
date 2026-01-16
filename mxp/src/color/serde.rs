use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};

use super::error::HexOutOfRangeError;
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
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        const EXPECT_INT: &str = "an integer between 0x000000 and 0xFFFFFF";

        macro_rules! impl_visit_int {
            ($t:ty, $i:ident, $unexpected:ident, $wide:ty) => {
                fn $i<E: de::Error>(self, v: $t) -> Result<Self::Value, E> {
                    let v = u32::try_from(v).map_err(|_| {
                        E::invalid_value(Unexpected::$unexpected(<$wide>::from(v)), &EXPECT_INT)
                    })?;
                    self.visit_u32(v)
                }
            };
        }
        macro_rules! impl_visit_signed {
            ($t:ty, $i:ident) => {
                impl_visit_int!($t, $i, Signed, i64);
            };
        }
        macro_rules! impl_visit_unsigned {
            ($t:ty, $i:ident) => {
                impl_visit_int!($t, $i, Unsigned, u64);
            };
        }

        macro_rules! impl_visit_128 {
            ($t:ty, $i:ident) => {
                fn $i<E: de::Error>(self, v: $t) -> Result<Self::Value, E> {
                    let value = u32::try_from(v).map_err(|_| {
                        E::invalid_value(Unexpected::Other(stringify!($t)), &EXPECT_INT)
                    })?;
                    self.visit_u32(value)
                }
            };
        }

        struct RgbColorVisitor;

        impl Visitor<'_> for RgbColorVisitor {
            type Value = RgbColor;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a hex color code, standard name, or integer between 0x000000 and 0xFFFFFF",
                )
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                RgbColor::named(v).ok_or_else(|| {
                    E::invalid_value(Unexpected::Str(v), &"a hex color code or standard name")
                })
            }

            fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E> {
                v.try_into().map_err(|code: HexOutOfRangeError| {
                    E::invalid_value(Unexpected::Unsigned(u64::from(code.0)), &EXPECT_INT)
                })
            }

            impl_visit_signed!(i8, visit_i8);
            impl_visit_signed!(i16, visit_i16);
            impl_visit_signed!(i32, visit_i32);
            impl_visit_signed!(i64, visit_i64);
            impl_visit_unsigned!(u8, visit_u8);
            impl_visit_unsigned!(u16, visit_u16);
            impl_visit_unsigned!(u64, visit_u64);
            impl_visit_128!(i128, visit_i128);
            impl_visit_128!(u128, visit_u128);
        }

        deserializer.deserialize_u32(RgbColorVisitor)
    }
}
