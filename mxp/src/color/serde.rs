use std::borrow::Cow;

use serde::de::{Error as _, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
        if deserializer.is_human_readable() {
            let code = Cow::<'de, str>::deserialize(deserializer)?;
            code.parse()
                .map_err(|_| D::Error::invalid_value(Unexpected::Str(&code), &"hex color code"))
        } else {
            u32::deserialize(deserializer)?
                .try_into()
                .map_err(|code: HexOutOfRangeError| {
                    D::Error::invalid_value(
                        Unexpected::Unsigned(u64::from(code.0)),
                        &"integer between 0x000000 and 0xFFFFFF",
                    )
                })
        }
    }
}
