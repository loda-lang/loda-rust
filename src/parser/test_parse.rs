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

    const INPUT_A000196: &str = r#"
    ; A000196: Integer part of square root of n.
    ; 0,1,1,1,2,2,2,2,2,3,3,3,3,3,3,3,4,4,4,4,4,4,4,4,4,5
    
    add $0,1
    mov $3,$0
    mul $3,-1
    lpb $0
      sub $3,1
      add $1,2
      sub $0,$1
    lpe
    div $1,2
    "#;

    const INPUT_A005131: &str = r#"
    ; A005131: A generalized continued fraction for Euler's number e.
    ; 1,0,1,1,2,1,1,4,1,1,6,1,1,8,1,1,10,1,1,12,1,1,14,1,1,16,1,1,18
    
    sub $0,1
    mov $1,$0
    lpb $0
      sub $0,3
      sub $1,1
    lpe
    add $1,1
    lpb $0
      div $0,2
      mov $1,2
    lpe
    sub $1,1
    "#;

    const INPUT_A002624: &str = r#"
    ; A002624: Expansion of (1-x)^(-3) * (1-x^2)^(-2).
    ; 1,3,8,16,30,50,80,120,175,245,336,448,588,756,960
    
    mov $12,$0
    mov $14,$0
    add $14,1
    lpb $14
      clr $0,12
      mov $0,$12
      sub $14,1
      sub $0,$14
      mov $9,$0
      mov $11,$0
      add $11,1
      lpb $11
        mov $0,$9
        sub $11,1
        sub $0,$11
        mov $6,$0
        add $6,4
        div $6,2
        bin $6,2
        add $10,$6
      lpe
      add $13,$10
    lpe
    mov $1,$13
    "#;

    const INPUT_A284429: &str = r#"
    ; A284429: A quasilinear solution to Hofstadter's Q recurrence.
    ; 2,1,3,5,1,3,8,1,3,11,1,3,14,1,3,17,1,3,20,1,3,23,1,3,26,1
    
    mov $1,$0
    mov $2,$0
    mod $2,3
    sub $2,4
    mov $0,$2
    div $0,2
    add $0,2
    add $1,3
    mov $3,4
    mov $4,-1
    pow $4,$2
    lpb $0
      sub $0,$0
      mov $1,$3
    lpe
    sub $0,$4
    sub $1,$0
    sub $1,2
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

        // Program that calls the A000079 program, and subtracts 1.
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

    #[test]
    fn test_10003_loop_restoring_previous_state_a000196() {
        let result = parse(INPUT_A000196);
        assert_eq!(result.is_ok(), true);
        let parse = result.unwrap();
        let program = parse.created_program.program;
        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(20);
        let expected: Vec<i64> = [0, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4].to_vec();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_loop_restoring_previous_state_a005131() {
        let result = parse(INPUT_A005131);
        assert_eq!(result.is_ok(), true);
        let parse = result.unwrap();
        let program = parse.created_program.program;
        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(20);
        let expected: Vec<i64> = [1, 0, 1, 1, 2, 1, 1, 4, 1, 1, 6, 1, 1, 8, 1, 1, 10, 1, 1, 12].to_vec();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10005_clear_memory_range() {
        let result = parse(INPUT_A002624);
        assert_eq!(result.is_ok(), true);
        let parse = result.unwrap();
        let program = parse.created_program.program;
        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(10);
        let expected: Vec<i64> = [1, 3, 8, 16, 30, 50, 80, 120, 175, 245].to_vec();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10006_use_of_power_instruction() {
        let result = parse(INPUT_A284429);
        assert_eq!(result.is_ok(), true);
        let parse = result.unwrap();
        let program = parse.created_program.program;
        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(10);
        let expected: Vec<i64> = [2, 1, 3, 5, 1, 3, 8, 1, 3, 11].to_vec();
        assert_eq!(actual, expected);
    }
}
