macro_rules! impl_parse_enum {
    ($t:ty, $v1:ident $(, $v:ident)* $(,)?) => {
        impl std::str::FromStr for $t {
            type Err = UnrecognizedVariant<Self>;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.eq_ignore_ascii_case(stringify!($v1)) {
                    Ok(Self::$v1)
                }
                $(
                    else if s.eq_ignore_ascii_case(stringify!($v)) {
                        Ok(Self::$v)
                    }
                )*
                else {
                    Err(Self::Err::new(s))
                }
            }
        }
    };
}

pub const fn to_ascii_lowercase<'a>(text: &[u8], buf: &'a mut [u8]) -> Option<&'a [u8]> {
    let Some((lower_buf, _)) = buf.split_at_mut_checked(text.len()) else {
        return None;
    };
    lower_buf.copy_from_slice(text);
    lower_buf.make_ascii_lowercase();
    Some(lower_buf)
}

macro_rules! match_ci {
    (
        $s:expr,
        $l:literal $(| $lo:literal)* => $i:expr,
        $(_ => $default:expr)? $(,)?
    ) => {
        if $s.eq_ignore_ascii_case($l) $(|| $s.eq_ignore_ascii_case($lo))* {
            $i
        } $(else {
            $default
        })?
    };
    (
        $s:expr,
        $l_first:literal $(| $lo_first:literal)* => $i_first:expr,
        $($l:literal $(| $lo:literal)* => $i:expr),*
        $(, _ => $default:expr)? $(,)?
    ) => {
        if $s.eq_ignore_ascii_case($l_first) $(|| $s.eq_ignore_ascii_case($lo_first))* {
            $i_first
        } $(else if $s.eq_ignore_ascii_case($l) $(|| $s.eq_ignore_ascii_case($lo))* {
            $i
        })* $(else {
            $default
        })?
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn match_ci_is_case_insensitive() {
        let result = match_ci! {"teSt",
            " TEST" => " TEST",
            "TEst" => "TEst",
            "teSt " => "teSt ",
            _ => "unmatched"
        };
        assert_eq!(result, "TEst");
    }
}
