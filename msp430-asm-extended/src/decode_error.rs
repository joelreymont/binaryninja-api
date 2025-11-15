use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecodeError {
    MissingInstruction,
    MissingOperand,
    InvalidOpcode(u16),
    InvalidJumpCondition(u16),
    InvalidAddressingMode,
    InvalidExtensionWord,
    InvalidRegisterCount,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInstruction => write!(f, "Not enough data for instruction"),
            Self::MissingOperand => write!(f, "Not enough data for operand"),
            Self::InvalidOpcode(opcode) => write!(f, "Invalid opcode: {:#x}", opcode),
            Self::InvalidJumpCondition(cond) => write!(f, "Invalid jump condition: {:#x}", cond),
            Self::InvalidAddressingMode => write!(f, "Invalid addressing mode"),
            Self::InvalidExtensionWord => write!(f, "Invalid extension word format"),
            Self::InvalidRegisterCount => write!(f, "Invalid register count for PUSHM/POPM"),
        }
    }
}

impl std::error::Error for DecodeError {}
