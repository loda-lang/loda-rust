
#[cfg(test)]
mod tests {
    use super::super::{Program, ProgramId, ProgramRunner, ProgramRunnerManager, ProgramSerializer, RegisterIndex};
    use crate::parser::{InstructionId, InstructionParameter, ParameterType};
    use super::super::node_calc::*;
    use super::super::node_loop_simple::*;
    use super::super::node_seq::*;
    
    fn node_calc(instruction_id: InstructionId, parameter0_type: ParameterType, parameter0_value: i64, parameter1_type: ParameterType, parameter1_value: i64) -> NodeCalc {
        let parameter0 = InstructionParameter::new(parameter0_type, parameter0_value);
        let parameter1 = InstructionParameter::new(parameter1_type, parameter1_value);
        let semantic_mode = NodeCalcSemanticMode::SmallLimits;
        NodeCalc::new(semantic_mode, instruction_id, parameter0, parameter1)
    }

    fn program_a000045() -> Program {
        // Fibonacci integer sequence
        // https://github.com/loda-lang/loda-programs/blob/main/oeis/000/A000045.asm
        let mut program_inner = Program::new();
        program_inner.push(node_calc(InstructionId::Subtract, ParameterType::Direct, 0, ParameterType::Constant, 1));
        program_inner.push(node_calc(InstructionId::Move, ParameterType::Direct, 2, ParameterType::Direct, 1));
        program_inner.push(node_calc(InstructionId::Add, ParameterType::Direct, 1, ParameterType::Direct, 3));
        program_inner.push(node_calc(InstructionId::Move, ParameterType::Direct, 3, ParameterType::Direct, 2));
    
        let mut program = Program::new();
        program.push(node_calc(InstructionId::Move, ParameterType::Direct, 3, ParameterType::Constant, 1));
        program.push(NodeLoopSimple::new(RegisterIndex(0), program_inner));
        program.push(node_calc(InstructionId::Move, ParameterType::Direct, 0, ParameterType::Direct, 1));
        program
    }

    const PROGRAM_A000045_SERIALIZED: &'static str =
r#"mov $3,1
lpb $0
  sub $0,1
  mov $2,$1
  add $1,$3
  mov $3,$2
lpe
mov $0,$1"#;

    #[test]
    fn test_10000_serialize() {
        let program = program_a000045();
        let mut pr = ProgramSerializer::new();
        program.serialize(&mut pr);
        assert_eq!(pr.to_string(), PROGRAM_A000045_SERIALIZED);
    }

    #[test]
    fn test_10001_run() {
        let program = program_a000045();
        let runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            program
        );
        assert_eq!(runner.inspect(10), "0,1,1,2,3,5,8,13,21,34");
    }

    #[test]
    fn test_10002_validate_call_nodes() {
        {
            // This program makes no calls to other programs
            let mut this_program = Program::new();
            this_program.push(node_calc(InstructionId::Add, ParameterType::Direct, 0, ParameterType::Direct, 0));
            this_program.push(node_calc(InstructionId::Subtract, ParameterType::Direct, 0, ParameterType::Constant, 1));

            // Programs without NodeCall cannot have problems with calls
            assert_eq!(this_program.validate_call_nodes().is_ok(), true);
        }
        {
            // This program calls another program (A000045)
            let mut this_program = Program::new();
            this_program.push(NodeSeq::new(InstructionParameter::new(ParameterType::Direct, 0), 45));

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
