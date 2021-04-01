
#[cfg(test)]
mod tests {
    use super::super::{Program, ProgramId, ProgramRunner, ProgramRunnerManager, ProgramSerializer, RegisterIndex, RegisterValue};
    use super::super::node_add::*;
    use super::super::node_call::*;
    use super::super::node_loop_simple::*;
    use super::super::node_move::*;
    use super::super::node_subtract::*;

    fn program_a000045() -> Program {
        // Fibonacci integer sequence
        // https://github.com/ckrause/loda/blob/master/programs/oeis/000/A000045.asm
        let mut program_inner = Program::new();
        program_inner.push(NodeSubtractConstant::new(RegisterIndex(0), RegisterValue::one()));
        program_inner.push(NodeMoveRegister::new(RegisterIndex(2), RegisterIndex(1)));
        program_inner.push(NodeAddRegister::new(RegisterIndex(1), RegisterIndex(3)));
        program_inner.push(NodeMoveRegister::new(RegisterIndex(3), RegisterIndex(2)));
    
        let mut program = Program::new();
        program.push(NodeMoveConstant::new(RegisterIndex(3), RegisterValue::one()));
        program.push(NodeLoopSimple::new(RegisterIndex(0), program_inner));
        program
    }

    const PROGRAM_A000045_SERIALIZED: &'static str =
r#"mov $3,1
lpb $0
  sub $0,1
  mov $2,$1
  add $1,$3
  mov $3,$2
lpe"#;


    #[test]
    fn test_10000_serialize() {
        let program = program_a000045();
        let mut pr = ProgramSerializer::new();
        program.serialize(&mut pr);
        assert_eq!(pr.to_string(), PROGRAM_A000045_SERIALIZED);
    }

    #[test]
    fn test_10001_accumulate_register_indexes() {
        let program = program_a000045();
        let mut register_vec: Vec<RegisterIndex> = vec!();
        program.accumulate_register_indexes(&mut register_vec);

        let mut indexes: Vec<u8> = register_vec.iter().map(|register_index| {
            register_index.0
        }).collect();
        indexes.sort();
        indexes.dedup();
        let expected: Vec<u8> = [0, 1, 2, 3].to_vec();
        assert_eq!(indexes, expected);
    }

    #[test]
    fn test_10002_max_register_index() {
        let program = program_a000045();
        let actual: u8 = program.max_register_index();
        assert_eq!(actual, 3);
    }

    #[test]
    fn test_10003_run() {
        let program = program_a000045();
        let runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            program
        );
        assert_eq!(runner.inspect(10), "0,1,1,2,3,5,8,13,21,34");
    }

    #[test]
    fn test_10004_validate_call_nodes() {
        {
            // This program makes no calls to other programs
            let mut this_program = Program::new();
            this_program.push(NodeAddRegister::new(RegisterIndex(0), RegisterIndex(0)));
            this_program.push(NodeSubtractConstant::new(RegisterIndex(0), RegisterValue::one()));

            // Programs without NodeCall cannot have problems with calls
            assert_eq!(this_program.validate_call_nodes().is_ok(), true);
        }
        {
            // This program calls another program (A000045)
            let mut this_program = Program::new();
            this_program.push(NodeCallConstant::new(RegisterIndex(0), 45));

            // Initially the NodeCall has no link with the A000045 program
            assert_eq!(this_program.validate_call_nodes().is_ok(), false);

            // Glue this program together with the A000045 program
            let runner0 = ProgramRunner::new(
                ProgramId::ProgramOEIS(45),
                program_a000045()
            );
            let mut pm = ProgramRunnerManager::new();
            pm.register(45, runner0);
            this_program.update_call(&mut pm);

            // Afterwards the NodeCall has a link to the A000045 program
            assert_eq!(this_program.validate_call_nodes().is_ok(), true);
        }
    }
}
