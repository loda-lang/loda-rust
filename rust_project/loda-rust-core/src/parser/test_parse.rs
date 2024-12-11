#[cfg(test)]
mod tests {
    use crate::execute::{Program, ProgramCache, ProgramId, ProgramRunner, ProgramRunnerManager};
    use crate::execute::node_calc::NodeCalcSemanticMode;
    use crate::unofficial_function::UnofficialFunctionRegistry;
    use crate::parser::{ParsedProgram, CreateProgram, ParseError};
    
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
    mov $0,$1
    "#;

    const INPUT_A000079: &str = r#"
    ; A000079: Powers of 2: a(n) = 2^n.
    ; 1,2,4,8,16,32,64,128,256,512
    
    mov $1,2
    pow $1,$0
    mov $0,$1
    "#;

    const INPUT_A000196: &str = r#"
    ; A000196: Integer part of square root of n.
    ; 0,1,1,1,2,2,2,2,2,3,3,3,3,3,3,3,4,4,4,4,4,4,4,4,4,5
    
    mov $1,1
    lpb $0
      add $1,2
      trn $0,$1
    lpe
    div $1,2
    mov $0,$1
    "#;

    const INPUT_A005131: &str = r#"
    ; A005131: A generalized continued fraction for Euler's number e.
    ; 1,0,1,1,2,1,1,4,1,1,6,1,1,8,1,1,10,1,1,12,1,1,14,1,1,16,1,1,18
    
    mul $0,2
    mov $2,2
    sub $2,$0
    sub $0,2
    add $2,3
    dif $2,3
    add $0,$2
    div $0,2
    "#;

    const INPUT_A007958: &str = r#"
    ; A007958: Even numbers with at least one odd digit.
    ; 10,12,14,16,18,30,32,34,36,38,50,52,54,56,58,70,72
    
    mov $1,1
    mov $2,6
    mov $6,$0
    lpb $1
      add $2,6
      add $6,1
      lpb $1
        sub $1,1
        add $2,2
      lpe
      add $2,2
    lpe
    lpb $5,5
      add $0,5
      trn $6,5
      lpb $5,3
        mov $6,$2
      lpe
    lpe
    lpb $0
      sub $0,1
      add $1,2
    lpe
    mov $0,$1
    "#;

    const INPUT_A206735: &str = r#"
    ; A206735: Triangle T(n,k), read by rows
    ; 1,0,1,0,2,1,0,3,3,1,0,4,6,4,1,0,5,10,10,5
    
    mov $4,$0
    lpb $4,$4
      add $3,1
      sub $4,$3
    lpe
    bin $3,$4
    mov $0,$3
    "#;

    const INPUT_A253472: &str = r#"
    ; A253472: Square Pairs
    ; 4,7,8,9,12,13,14,15,16,17,18,19,20
    
    mov $2,$0
    lpb $2,$0
      add $0,2
      mov $5,$2
      sub $2,$5
    lpe
    add $0,4
    mov $1,$0
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
    mov $0,$1
    "#;

    /// This function can parse simple programs, without the `seq` instruction.
    /// This function does not resolve dependencies.
    fn parse(input: &str) -> Result<Program, ParseError> {
      let parsed_program: ParsedProgram = ParsedProgram::parse_program(input)?;
      let create_program = CreateProgram::new(NodeCalcSemanticMode::Unlimited);
      let unofficial_function_registry = UnofficialFunctionRegistry::new();
      let program: Program = create_program.create_program(&parsed_program, &unofficial_function_registry)?;
      Ok(program)
    }

    #[test]
    fn test_10000_fibonacci() {
        let program: Program = parse(INPUT_A000045).expect("program");
        let runner = ProgramRunner::new(
          ProgramId::ProgramOEIS(45),
          program
        );
        assert_eq!(runner.inspect(10), "0,1,1,2,3,5,8,13,21,34");
    }

    #[test]
    fn test_10001_powers_of_2() {
        let program: Program = parse(INPUT_A000079).expect("program");
        let runner = ProgramRunner::new(
          ProgramId::ProgramOEIS(79),
          program
        );
        assert_eq!(runner.inspect(10), "1,2,4,8,16,32,64,128,256,512");
    }

    fn program_that_calls_another_program() -> ProgramRunner {
        let program0: Program = parse(INPUT_A000079).expect("program");
        let runner0 = ProgramRunner::new(
            ProgramId::ProgramOEIS(79),
            program0
        );

        let mut pm = ProgramRunnerManager::new();
        pm.register(79, runner0);

        // Program that calls the A000079 program, and subtracts 1.
        let input = r#"
        seq $0,79
        sub $0,1
        "#;
    
        let mut program: Program = parse(input).expect("program");

        // Obtain a list of dependencies.
        let mut program_id_vec: Vec<u64> = vec!();
        program.accumulate_call_dependencies(&mut program_id_vec);
        assert_eq!(program_id_vec.len(), 1);
        let dependency0: u64 = program_id_vec.first().map_or(0, |m| m.clone());
        assert_eq!(dependency0, 79);

        // Glue A000079 together with this program.
        program.update_call(&mut pm);

        ProgramRunner::new(
          ProgramId::ProgramWithoutId,
          program
        )
    }

    #[test]
    fn test_10002_call_other_programs() {
        let runner: ProgramRunner = program_that_calls_another_program();
        assert_eq!(runner.inspect(10), "0,1,3,7,15,31,63,127,255,511");
    }

    #[test]
    fn test_10003_caching_of_computed_results() {
        let runner: ProgramRunner = program_that_calls_another_program();

        let mut cache = ProgramCache::new();
        assert_eq!(cache.hit_miss_info(), "hit:0 miss:0,0");

        let actual0: String = runner.inspect_advanced(10, &mut cache);
        assert_eq!(actual0, "0,1,3,7,15,31,63,127,255,511");
        assert_eq!(cache.hit_miss_info(), "hit:0 miss:10,10");

        let actual1: String = runner.inspect_advanced(10, &mut cache);
        assert_eq!(actual1, "0,1,3,7,15,31,63,127,255,511");
        assert_eq!(cache.hit_miss_info(), "hit:10 miss:10,20");

        let actual2: String = runner.inspect_advanced(10, &mut cache);
        assert_eq!(actual2, "0,1,3,7,15,31,63,127,255,511");
        assert_eq!(cache.hit_miss_info(), "hit:20 miss:10,30");
    }

    #[test]
    fn test_10004_loop_restoring_previous_state_a000196() {
        let program: Program = parse(INPUT_A000196).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramOEIS(196),
            program
        );
        assert_eq!(runner.inspect(20), "0,1,1,1,2,2,2,2,2,3,3,3,3,3,3,3,4,4,4,4");
    }

    #[test]
    fn test_10005_loop_restoring_previous_state_a005131() {
        let program: Program = parse(INPUT_A005131).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramOEIS(5131),
            program
        );
        assert_eq!(runner.inspect(20), "1,0,1,1,2,1,1,4,1,1,6,1,1,8,1,1,10,1,1,12");
    }

    #[test]
    fn test_10006_use_of_power_instruction() {
        let program: Program = parse(INPUT_A284429).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramOEIS(284429),
            program
        );
        assert_eq!(runner.inspect(10), "2,1,3,5,1,3,8,1,3,11");
    }

    #[test]
    fn test_10007_use_of_loop_with_contant_greater_than_one() {
        let program: Program = parse(INPUT_A007958).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramOEIS(7958),
            program
        );
        assert_eq!(runner.inspect(15), "10,12,14,16,18,30,32,34,36,38,50,52,54,56,58");
    }

    #[test]
    fn test_10008_use_of_loop_with_range_length_from_register1() {
        let program: Program = parse(INPUT_A253472).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramOEIS(253472),
            program
        );
        assert_eq!(runner.inspect(15), "4,7,8,9,12,13,14,15,16,17,18,19,20,21,22");
    }

    #[test]
    fn test_10009_use_of_loop_with_range_length_from_register2() {
        let program: Program = parse(INPUT_A206735).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramOEIS(206735),
            program
        );
        assert_eq!(runner.inspect(15), "1,0,1,0,2,1,0,3,3,1,0,4,6,4,1");
    }

    #[test]
    fn test_20000_unofficial_loop_subtract_instruction_normal() {
      let input = r#"
        mov $1,0 ; sum
        lps $0
          add $1,$0 ; add to sum
        lpe
        mov $0,$1
        "#;
        let program: Program = parse(input).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            program
        );
        assert_eq!(runner.inspect(6), "0,1,3,6,10,15");
    }

    #[test]
    fn test_20001_unofficial_loop_subtract_instruction_break_in_first_iteration() {
      let input = r#"
        mov $2,0 ; iteration counter
        lps $0
          add $2,1 ; increment iteration counter
          mov $0,0 ; break the loop immediately
        lpe
        mov $0,$2
        "#;
        let program: Program = parse(input).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            program
        );
        assert_eq!(runner.inspect(6), "0,1,1,1,1,1");
    }

    #[test]
    fn test_20002_unofficial_loop_subtract_instruction_break_in_second_iteration() {
      let input = r#"
        mov $2,0 ; iteration counter
        lps $0
          add $2,1 ; increment iteration counter
          mov $3,$2
          cmp $3,2 ; index where to reset
          cmp $3,0
          mul $0,$3 ; continue while 1, break when 0
        lpe
        mov $0,$2
        "#;
        let program: Program = parse(input).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            program
        );
        assert_eq!(runner.inspect(6), "0,1,2,2,2,2");
    }

    #[test]
    fn test_20003_unofficial_loop_subtract_instruction_break_when_setting_a_higher_value() {
      let input = r#"
        mov $2,0 ; iteration counter
        lps $0
          add $2,1 ; increment iteration counter
          mov $3,$2
          cmp $3,3 ; index where to reset
          add $0,$3 ; continue while 0, break when setting a value that is higher
        lpe
        mov $0,$2
        "#;
        let program: Program = parse(input).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            program
        );
        assert_eq!(runner.inspect(6), "0,1,2,3,3,3");
    }

    #[test]
    fn test_20004_unofficial_loop_subtract_instruction_break_when_setting_a_lower_value() {
      let input = r#"
        mov $2,0 ; iteration counter
        lps $0
          add $2,1 ; increment iteration counter
          mov $3,$2
          cmp $3,4 ; index where to reset
          sub $0,$3 ; continue while 0, break when setting a value that is lower
        lpe
        mov $0,$2
        "#;
        let program: Program = parse(input).expect("program");
        let runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            program
        );
        assert_eq!(runner.inspect(7), "0,1,2,3,4,4,4");
    }
}
