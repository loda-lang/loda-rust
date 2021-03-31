mod instruction;
mod instruction_id;
mod parameter_type;
mod parse_parameters;
mod parse_program;

pub use instruction::{Instruction, InstructionParameter};
pub use instruction_id::{InstructionId, ParseInstructionIdError};
pub use parameter_type::ParameterType;
pub use parse_parameters::{ParseParametersError, parse_parameters};
pub use parse_program::{ParsedProgram, ParseProgramError, parse_program};

pub mod create_program;
pub mod extract_parameter_re;
pub mod extract_row_re;
pub mod parse;
pub mod remove_comment;
pub mod validate_loops;
pub mod test_parse;
