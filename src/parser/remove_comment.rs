use regex::Regex;
use lazy_static::lazy_static;
use std::borrow::Cow;

// Remove a "; comment" from an instruction row of an assembler program
pub fn remove_comment(input: &str) -> Cow<str> {
    lazy_static! {
        static ref REMOVE_COMMENT_RE: Regex = Regex::new(
            "[;].*$"
        ).unwrap();
    }
    REMOVE_COMMENT_RE.replace(input, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    static INPUT: &'static [&'static str] = &[
        "; comment",
        ";; ; ; comment ;",
        " ; comment",
        " ",
        "a;; comment",
        "b ",
        " c ",
    ];

    static OUTPUT: &'static [&'static str] = &[
        "",
        "",
        " ",
        " ",
        "a",
        "b ",
        " c ",
    ];

    fn process<S: AsRef<str>>(input: S) -> String {
        let input = input.as_ref();
        remove_comment(input).to_string()
    }

    #[test]
    fn it_works() {
        for (index, input) in INPUT.iter().enumerate() {
            assert_eq!(process(input), OUTPUT[index]);
        }
    }
}
