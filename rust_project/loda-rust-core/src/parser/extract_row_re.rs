use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// Extract the instruction and its parameters, from a row of assembler code
    pub static ref EXTRACT_ROW_RE: Regex = Regex::new(
        "^[ \t]*([a-z]{2,5}\\b)(?:[ \t]+(.*))?$"
    ).unwrap();
}

#[cfg(test)]
mod tests {
    static INPUT: &'static [&'static str] = &[
        "toolonginstruction $1",
        "",
        " ",
        "mov0 1,2",
        "mov$0 1,2",
        "80s",
        "boom",
        "mov $5,1 ",
        "lpb $0",
        "  sub $0,1 ",
        "  add $0, -5",
        "  add $1,   $2  ,$3",
        "  xyz   -1,-2,-3",
        "  mov $1,$1",
        "lpe",
        "mov $1,$0",
        "\tmov\t$3,$10",
        "mov $3\t,\t$10\t",
    ];

    static OUTPUT: &'static [&'static str] = &[
        "MISMATCH",
        "MISMATCH",
        "MISMATCH",
        "MISMATCH",
        "MISMATCH",
        "MISMATCH",
        "boom:",
        "mov:$5,1 ",
        "lpb:$0",
        "sub:$0,1 ",
        "add:$0, -5",
        "add:$1,   $2  ,$3",
        "xyz:-1,-2,-3",
        "mov:$1,$1",
        "lpe:",
        "mov:$1,$0",
        "mov:$3,$10",
        "mov:$3\t,\t$10\t",
    ];

    fn process<S: AsRef<str>>(input: S) -> String {
        let input = input.as_ref();

        let captures = match super::EXTRACT_ROW_RE.captures(input) {
            Some(value) => value,
            None => {
                return "MISMATCH".to_string();
            }
        };

        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
        format!("{}:{}", capture1, capture2)
    }

    #[test]
    fn it_works() {
        for (index, input) in INPUT.iter().enumerate() {
            assert_eq!(process(input), OUTPUT[index]);
        }
    }
}
