use super::RgbColor;

use std::str;

pub(crate) struct RgbDigits<const N: usize>([u8; N]);

const fn hex_digit(value: u8, a: u8) -> u8 {
    if value < 10 {
        value + b'0'
    } else {
        value - 10 + a
    }
}

const fn hex_upper(value: u8) -> u8 {
    hex_digit(value, b'A')
}

const fn hex_lower(value: u8) -> u8 {
    hex_digit(value, b'a')
}

impl RgbDigits<7> {
    pub const fn prefixed(color: RgbColor) -> Self {
        Self([
            b'#',
            hex_upper(color.r >> 4),
            hex_upper(color.r & 0xF),
            hex_upper(color.g >> 4),
            hex_upper(color.g & 0xF),
            hex_upper(color.b >> 4),
            hex_upper(color.b & 0xF),
        ])
    }
}

impl RgbDigits<6> {
    pub const fn upper(color: RgbColor) -> Self {
        Self([
            hex_upper(color.r >> 4),
            hex_upper(color.r & 0xF),
            hex_upper(color.g >> 4),
            hex_upper(color.g & 0xF),
            hex_upper(color.b >> 4),
            hex_upper(color.b & 0xF),
        ])
    }

    pub const fn lower(color: RgbColor) -> Self {
        Self([
            hex_lower(color.r >> 4),
            hex_lower(color.r & 0xF),
            hex_lower(color.g >> 4),
            hex_lower(color.g & 0xF),
            hex_lower(color.b >> 4),
            hex_lower(color.b & 0xF),
        ])
    }
}

impl<const N: usize> RgbDigits<N> {
    pub const fn as_str(&self) -> &str {
        // SAFETY: all digits are valid ASCII
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}
