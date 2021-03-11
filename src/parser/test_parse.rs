#[cfg(test)]
mod tests {
    use crate::execute::{ProgramRunner,ProgramRunnerManager};
    use super::super::parse::*;
    
    const INPUT_A000045: &str = r#"
    ; A000045: Fibonacci numbers
    ; 0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,2584,4181,6765
    
    mov $3,1
    lpb $0
      sub $0,1
      mov $2,$1
      add $1,$3
      mov $3,$2
    lpe
    "#;

    const INPUT_A000079: &str = r#"
    ; A000079: Powers of 2: a(n) = 2^n.
    ; 1,2,4,8,16,32,64,128,256,512
    
    mov $1,2
    pow $1,$0
    "#;

    #[test]
    fn test_10000_fibonacci() {
        let result = parse(INPUT_A000045);
        assert_eq!(result.is_ok(), true);
        let parse = result.unwrap();
        let program = parse.created_program.program;
        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(10);
        let expected: Vec<i64> = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34].to_vec();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_powers_of_2() {
        let result = parse(INPUT_A000079);
        assert_eq!(result.is_ok(), true);
        let parse = result.unwrap();
        let program = parse.created_program.program;
        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(10);
        let expected: Vec<i64> = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512].to_vec();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_call_other_programs() {
        let result0 = parse(INPUT_A000079);
        assert_eq!(result0.is_ok(), true);
        let parse0 = result0.unwrap();
        let program0 = parse0.created_program.program;
        let runner0 = ProgramRunner::new(program0);

        let mut pm = ProgramRunnerManager::new();
        pm.register(79, runner0);

        let input = r#"
        cal $0,79
        sub $0,1
        mov $1,$0
        "#;
    
        let result1 = parse(input);
        assert_eq!(result1.is_ok(), true);
        let parse1 = result1.unwrap();

        let mut program = parse1.created_program.program;

        // Obtain a list of dependencies.
        let mut program_id_vec: Vec<u64> = vec!();
        program.accumulate_call_dependencies(&mut program_id_vec);
        assert_eq!(program_id_vec.len(), 1);
        let dependency0: u64 = program_id_vec.first().map_or(0, |m| m.clone());
        assert_eq!(dependency0, 79);

        // Glue A000079 together with this program.
        program.update_call(&mut pm);

        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(10);
        let expected: Vec<i64> = [0, 1, 3, 7, 15, 31, 63, 127, 255, 511].to_vec();
        assert_eq!(actual, expected);
    }
}
