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
