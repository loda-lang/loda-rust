//! Translate from LODA source code to a program instance.
mod create_program;
mod extract_row_re;
mod instruction;
mod instruction_id;
mod instruction_parameter;
mod parameter_type;
mod parse_error;
mod parse_parameters;
mod parse_program;
mod remove_comment;

pub use create_program::{CreateProgramError, CreateProgram};
pub use extract_row_re::EXTRACT_ROW_RE;
pub use instruction::Instruction;
pub use instruction_id::{InstructionId, ParseInstructionIdError};
pub use instruction_parameter::InstructionParameter;
pub use parameter_type::ParameterType;
pub use parse_error::ParseError;
pub use parse_parameters::{ParseParametersError, parse_parameters};
pub use parse_program::{ParsedProgram, ParseProgramError};
pub use remove_comment::remove_comment;

pub mod extract_parameter_re;
pub mod validate_loops;
pub mod test_parse;
