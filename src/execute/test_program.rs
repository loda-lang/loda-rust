
#[cfg(test)]
mod tests {
    use super::super::{RegisterIndex,RegisterValue,Program,ProgramRunner};
    use super::super::node_add::*;
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

    #[test]
    fn test_10000_accumulate_register_indexes() {
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
    fn test_10001_max_register_index() {
        let program = program_a000045();
        let actual: u8 = program.max_register_index();
        assert_eq!(actual, 3);
    }

    #[test]
    fn test_10002_run() {
        let program = program_a000045();
        let runner = ProgramRunner::new(program);
        let actual: Vec<i64> = runner.run_terms(10);
        let expected: Vec<i64> = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34].to_vec();
        assert_eq!(actual, expected);
    }
}
