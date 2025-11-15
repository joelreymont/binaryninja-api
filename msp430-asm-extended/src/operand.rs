use crate::decode_error::DecodeError;
use crate::extension_word::ExtensionWord;
use std::fmt;

/// Operand width - Byte (.B), Word (.W), or Address (.A for 20-bit)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperandWidth {
    Byte,
    Word,
    Address, // 20-bit for MSP430X
}

impl From<u8> for OperandWidth {
    fn from(value: u8) -> Self {
        match value {
            0 => OperandWidth::Word,
            1 => OperandWidth::Byte,
            2 => OperandWidth::Address,
            _ => OperandWidth::Word, // Default to word
        }
    }
}

impl fmt::Display for OperandWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperandWidth::Byte => write!(f, ".b"),
            OperandWidth::Word => write!(f, ".w"),
            OperandWidth::Address => write!(f, ".a"),
        }
    }
}

/// Operand types for MSP430/MSP430X
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    /// Register direct: Rn
    RegisterDirect(u8),

    /// Indexed: X(Rn) - 16-bit offset
    Indexed((u8, i16)),

    /// Indexed 20-bit: X(Rn) - 20-bit offset for MSP430X
    Indexed20((u8, i32)),

    /// Register indirect: @Rn
    RegisterIndirect(u8),

    /// Register indirect with auto-increment: @Rn+
    RegisterIndirectAutoIncrement(u8),

    /// Symbolic: ADDR (PC-relative) - 16-bit
    Symbolic(i16),

    /// Symbolic 20-bit: ADDR (PC-relative) - 20-bit for MSP430X
    Symbolic20(i32),

    /// Immediate: #N - 16-bit
    Immediate(u16),

    /// Immediate 20-bit: #N - 20-bit for MSP430X
    Immediate20(u32),

    /// Absolute: &ADDR - 16-bit
    Absolute(u16),

    /// Absolute 20-bit: &ADDR - 20-bit for MSP430X
    Absolute20(u32),

    /// Constant (from constant generator)
    Constant(i8),
}

impl Operand {
    /// Returns the size in bytes of operand data (not including the instruction word)
    pub fn size(&self) -> usize {
        match self {
            Operand::RegisterDirect(_)
            | Operand::RegisterIndirect(_)
            | Operand::RegisterIndirectAutoIncrement(_)
            | Operand::Constant(_) => 0,
            Operand::Indexed(_)
            | Operand::Indexed20(_)
            | Operand::Symbolic(_)
            | Operand::Symbolic20(_)
            | Operand::Immediate(_)
            | Operand::Immediate20(_)
            | Operand::Absolute(_)
            | Operand::Absolute20(_) => 2,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::RegisterDirect(r) => write!(f, "r{}", r),
            Operand::Indexed((r, offset)) => write!(f, "{}(r{})", offset, r),
            Operand::Indexed20((r, offset)) => write!(f, "{:#x}(r{})", offset, r),
            Operand::RegisterIndirect(r) => write!(f, "@r{}", r),
            Operand::RegisterIndirectAutoIncrement(r) => write!(f, "@r{}+", r),
            Operand::Symbolic(offset) => write!(f, "{}", offset),
            Operand::Symbolic20(offset) => write!(f, "{:#x}", offset),
            Operand::Immediate(val) => write!(f, "#{:#x}", val),
            Operand::Immediate20(val) => write!(f, "#{:#x}", val),
            Operand::Absolute(addr) => write!(f, "&{:#x}", addr),
            Operand::Absolute20(addr) => write!(f, "&{:#x}", addr),
            Operand::Constant(val) => write!(f, "#{}", val),
        }
    }
}

/// Parse source operand with optional extension word
pub fn parse_source<'a>(
    register: u8,
    addressing_mode: u16,
    data: &'a [u8],
    extension: Option<&ExtensionWord>,
) -> Result<(Operand, &'a [u8]), DecodeError> {
    match addressing_mode {
        // Register direct
        0 => Ok((Operand::RegisterDirect(register), data)),

        // Indexed or Symbolic or Absolute or Immediate
        1 => {
            if data.len() < 2 {
                return Err(DecodeError::MissingOperand);
            }

            let (offset_bytes, remaining) = data.split_at(2);
            let offset = i16::from_le_bytes([offset_bytes[0], offset_bytes[1]]);

            match register {
                // PC: Symbolic or Immediate
                0 => {
                    if let Some(ext) = extension {
                        // MSP430X: 20-bit immediate
                        let imm20 = ext.extend_source(offset as u16);
                        Ok((Operand::Immediate20(imm20), remaining))
                    } else {
                        // Symbolic (PC-relative)
                        Ok((Operand::Symbolic(offset), remaining))
                    }
                }
                // SR: Absolute or Immediate
                2 => {
                    if let Some(ext) = extension {
                        // MSP430X: 20-bit absolute
                        let abs20 = ext.extend_source(offset as u16);
                        Ok((Operand::Absolute20(abs20), remaining))
                    } else {
                        // Absolute addressing
                        Ok((Operand::Absolute(offset as u16), remaining))
                    }
                }
                // Other: Indexed
                _ => {
                    if let Some(ext) = extension {
                        // MSP430X: 20-bit indexed
                        let idx20 = ext.extend_source(offset as u16) as i32;
                        Ok((Operand::Indexed20((register, idx20)), remaining))
                    } else {
                        Ok((Operand::Indexed((register, offset)), remaining))
                    }
                }
            }
        }

        // Register indirect
        2 => {
            // Check for constant generator special cases
            match register {
                2 => Ok((Operand::Constant(4), data)), // SR with As=10 = #4
                3 => Ok((Operand::Constant(2), data)), // CG with As=10 = #2
                _ => Ok((Operand::RegisterIndirect(register), data)),
            }
        }

        // Register indirect auto-increment or Immediate
        3 => {
            match register {
                // PC with As=11: Immediate mode
                0 => {
                    if data.len() < 2 {
                        return Err(DecodeError::MissingOperand);
                    }

                    let (imm_bytes, remaining) = data.split_at(2);
                    let imm = u16::from_le_bytes([imm_bytes[0], imm_bytes[1]]);

                    if let Some(ext) = extension {
                        // MSP430X: 20-bit immediate
                        let imm20 = ext.extend_source(imm);
                        Ok((Operand::Immediate20(imm20), remaining))
                    } else {
                        Ok((Operand::Immediate(imm), remaining))
                    }
                }
                // SR: Constant #8
                2 => Ok((Operand::Constant(8), data)),
                // CG: Constant #0
                3 => Ok((Operand::Constant(0), data)),
                // Other: Register indirect with auto-increment
                _ => Ok((Operand::RegisterIndirectAutoIncrement(register), data)),
            }
        }

        _ => Err(DecodeError::InvalidAddressingMode),
    }
}

/// Parse destination operand with optional extension word
pub fn parse_destination(
    register: u8,
    ad_bit: u16,
    data: &[u8],
    extension: Option<&ExtensionWord>,
) -> Result<Operand, DecodeError> {
    match ad_bit {
        // Register direct
        0 => Ok(Operand::RegisterDirect(register)),

        // Indexed, Symbolic, or Absolute
        1 => {
            if data.len() < 2 {
                return Err(DecodeError::MissingOperand);
            }

            let offset = i16::from_le_bytes([data[0], data[1]]);

            match register {
                // PC: Symbolic
                0 => {
                    if let Some(ext) = extension {
                        let sym20 = ext.extend_dest(offset as u16) as i32;
                        Ok(Operand::Symbolic20(sym20))
                    } else {
                        Ok(Operand::Symbolic(offset))
                    }
                }
                // SR: Absolute
                2 => {
                    if let Some(ext) = extension {
                        let abs20 = ext.extend_dest(offset as u16);
                        Ok(Operand::Absolute20(abs20))
                    } else {
                        Ok(Operand::Absolute(offset as u16))
                    }
                }
                // Other: Indexed
                _ => {
                    if let Some(ext) = extension {
                        let idx20 = ext.extend_dest(offset as u16) as i32;
                        Ok(Operand::Indexed20((register, idx20)))
                    } else {
                        Ok(Operand::Indexed((register, offset)))
                    }
                }
            }
        }

        _ => Err(DecodeError::InvalidAddressingMode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_register_direct() {
        let data = [];
        let result = parse_source(15, 0, &data, None);
        assert_eq!(result, Ok((Operand::RegisterDirect(15), &[][..])));
    }

    #[test]
    fn parse_immediate_16bit() {
        let data = [0x34, 0x12]; // 0x1234
        let result = parse_source(0, 3, &data, None);
        assert_eq!(result, Ok((Operand::Immediate(0x1234), &[][..])));
    }

    #[test]
    fn parse_constant_generator() {
        // CG (R3) with As=11 = constant 0
        let data = [];
        let result = parse_source(3, 3, &data, None);
        assert_eq!(result, Ok((Operand::Constant(0), &[][..])));

        // SR (R2) with As=10 = constant 4
        let result = parse_source(2, 2, &data, None);
        assert_eq!(result, Ok((Operand::Constant(4), &[][..])));
    }
}
