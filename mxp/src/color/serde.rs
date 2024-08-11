use super::rgb::RgbColor;
use super::world_color::WorldColor;
use serde::de::{Error as _, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "serde")]
impl Serialize for RgbColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("#{:0>6X}", self.code()))
        } else {
            serializer.serialize_u32(self.code())
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for RgbColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let code = <&str>::deserialize(deserializer)?;
            code.parse()
                .map_err(|_| D::Error::invalid_value(Unexpected::Str(code), &"hex color code"))
        } else {
            let code = u32::deserialize(deserializer)?;
            if code <= 0xFFFFFF {
                Ok(Self::hex(code))
            } else {
                Err(D::Error::invalid_value(
                    Unexpected::Unsigned(code as u64),
                    &"integer between 0x000000 and 0xFFFFFF",
                ))
            }
        }
    }
}

impl Serialize for WorldColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            Self::Ansi(code) if serializer.is_human_readable() => {
                serializer.serialize_str(&code.to_string())
            }
            Self::Ansi(code) => serializer.serialize_u32(0xFFFFFF + 1 + code as u32),
            Self::Rgb(color) => color.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for WorldColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let code = <&str>::deserialize(deserializer)?;
            if code.starts_with('#') {
                match code.parse() {
                    Ok(hex) => Ok(Self::Rgb(hex)),
                    Err(_) => Err(D::Error::invalid_value(
                        Unexpected::Str(code),
                        &"hex color code or stringified integer between 0 and 255",
                    )),
                }
            } else {
                match code.parse() {
                    Ok(ansi) => Ok(Self::Ansi(ansi)),
                    Err(_) => Err(D::Error::invalid_value(
                        Unexpected::Str(code),
                        &"hex color code or stringified integer between 0 and 255",
                    )),
                }
            }
        } else {
            let code = u32::deserialize(deserializer)?;
            if code <= 0xFFFFFF {
                Ok(Self::Rgb(RgbColor::hex(code)))
            } else if code <= 0xFFFFFF + 16 {
                Ok(Self::Ansi((code - 0xFFFFFF - 1) as u8))
            } else {
                Err(D::Error::invalid_value(
                    Unexpected::Unsigned(code as u64),
                    &"integer between 0x000000 and 0x100000F",
                ))
            }
        }
    }
}
