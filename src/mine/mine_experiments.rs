use super::check_fixed_length_sequence::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    impl CheckFixedLengthSequence {
        pub fn mock_new() -> Self {
            panic!("not implemented");
        }
    }    
/*
    #[test]
    fn test_10000_mining() {
        let checker0 = CheckFixedLengthSequence::mock_new();
        let checker1 = CheckFixedLengthSequence::mock_new();
        let mut program = generate_program();
        for _ in 0..100 {
            if let Err(err) = program.parse() {
                program.mutate_erroneous_lines_to_fix_problem(err);
                continue;
            }
            let vec0 = match program.terms(10) {
                Ok(value) => value,
                Err(err) => {
                    program.mutate_erroneous_lines_to_fix_problem(err);
                    continue;
                }
            };
            if !checker0.check(vec0) {
                program.mutate();
                continue;
            }
            let vec1 = match program.terms(20) {
                Ok(value) => value,
                Err(err) => {
                    program.mutate_erroneous_lines_to_fix_problem(err);
                    continue;
                }
            };
            if !checker1.check(vec1) {
                program.mutate();
                continue;
            }
            println!("found a candidate program: {}", program);
            break;
        }
    }

    */
}
