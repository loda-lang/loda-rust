use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// Extract one parameter from an instruction row
    pub static ref EXTRACT_PARAMETER_RE: Regex = Regex::new(
        "^([$]*)(-?\\d+)$"
    ).unwrap();
}

#[cfg(test)]
mod tests {
    static INPUT: &'static [&'static str] = &[
        "junk",
        "$x",
        "9x",
        "-x",
        "",
        " ",
        "-0",
        "-0000",
        "$0 ",
        " $0",
        " $-0 ",
        " $$0",
        " $$$$$-1234",
        " $10,-666",
        " $3,1 ",
        " $2,$1",
        "  $3,$2",
        " $1,   $3",
        "  $-5,$-5 , $-5",
    ];

    static OUTPUT: &'static [&'static str] = &[
        "MISMATCH",
        "MISMATCH",
        "MISMATCH",
        "MISMATCH",
        "EMPTY",
        "EMPTY",
        "-0",
        "-0000",
        "$0",
        "$0",
        "$-0",
        "$$0",
        "$$$$$-1234",
        "$10SEP-666",
        "$3SEP1",
        "$2SEP$1",
        "$3SEP$2",
        "$1SEP$3",
        "$-5SEP$-5SEP$-5",
    ];

    fn process<S: AsRef<str>>(input: S) -> String {
        let input = input.as_ref();

        let re = &super::EXTRACT_PARAMETER_RE;
        let split = input.split(",");
        let mut result: Vec<String> = vec!();
        for split_item in split {
            let trimmed_split_item = split_item.trim();
            if trimmed_split_item.is_empty() {
                result.push("EMPTY".to_string());
                continue;
            }

            let captures = match re.captures(trimmed_split_item) {
                Some(value) => value,
                None => {
                    result.push("MISMATCH".to_string());
                    continue;
                }
            };
    
            let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
            let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
            let s = format!("{}{}", capture1, capture2);
            result.push(s);
        }
        result.join("SEP")
    }

    #[test]
    fn it_works() {
        for (index, input) in INPUT.iter().enumerate() {
            assert_eq!(process(input), OUTPUT[index]);
        }
    }
}
