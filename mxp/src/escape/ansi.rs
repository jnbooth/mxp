pub const RESET: u8 = 0;
pub const BOLD: u8 = 1;
pub const BLINK: u8 = 3;
pub const UNDERLINE: u8 = 4;
pub const SLOW_BLINK: u8 = 5;
pub const FAST_BLINK: u8 = 6;
pub const INVERSE: u8 = 7;
pub const STRIKEOUT: u8 = 9;

pub const CANCEL_BOLD: u8 = 22;
pub const CANCEL_BLINK: u8 = 23;
pub const CANCEL_UNDERLINE: u8 = 24;
pub const CANCEL_SLOW_BLINK: u8 = 25;
pub const CANCEL_FAST_BLINK: u8 = 26;
pub const CANCEL_INVERSE: u8 = 27;
pub const CANCEL_STRIKEOUT: u8 = 29;

pub const FG_BLACK: u8 = 30;
pub const FG_RED: u8 = 31;
pub const FG_GREEN: u8 = 32;
pub const FG_YELLOW: u8 = 33;
pub const FG_BLUE: u8 = 34;
pub const FG_MAGENTA: u8 = 35;
pub const FG_CYAN: u8 = 36;
pub const FG_WHITE: u8 = 37;
pub const FG_256_COLOR: u8 = 38;
pub const FG_DEFAULT: u8 = 39;

pub const BG_BLACK: u8 = 40;
pub const BG_RED: u8 = 41;
pub const BG_GREEN: u8 = 42;
pub const BG_YELLOW: u8 = 43;
pub const BG_BLUE: u8 = 44;
pub const BG_MAGENTA: u8 = 45;
pub const BG_CYAN: u8 = 46;
pub const BG_WHITE: u8 = 47;
pub const BG_256_COLOR: u8 = 48;
pub const BG_DEFAULT: u8 = 49;

#[inline]
#[must_use = "inputs are not modified"]
pub const fn append_digit_to_code(code: u8, digit: u8) -> u8 {
    code * 10 + (digit - b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_digits_to_code() {
        let mut code = 1;
        code = append_digit_to_code(code, b'0');
        code = append_digit_to_code(code, b'9');
        assert_eq!(code, 109);
    }
}
