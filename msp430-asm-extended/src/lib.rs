pub mod decode_error;
pub mod emulate;
pub mod extension_word;
pub mod instruction;
pub mod jxx;
pub mod msp430x_instructions;
pub mod operand;
pub mod single_operand;
pub mod two_operand;

use decode_error::DecodeError;
use emulate::Emulate;
use extension_word::ExtensionWord;
use instruction::Instruction;
use jxx::*;
use msp430x_instructions::*;
use operand::{parse_destination, parse_source, OperandWidth};
use single_operand::*;
use two_operand::*;

// Base MSP430 instruction constants
const RRC_OPCODE: u16 = 0;
const SWPB_OPCODE: u16 = 1;
const RRA_OPCODE: u16 = 2;
const SXT_OPCODE: u16 = 3;
const PUSH_OPCODE: u16 = 4;
const CALL_OPCODE: u16 = 5;
const RETI_OPCODE: u16 = 6;

const MOV_OPCODE: u16 = 4;
const ADD_OPCODE: u16 = 5;
const ADDC_OPCODE: u16 = 6;
const SUBC_OPCODE: u16 = 7;
const SUB_OPCODE: u16 = 8;
const CMP_OPCODE: u16 = 9;
const DADD_OPCODE: u16 = 10;
const BIT_OPCODE: u16 = 11;
const BIC_OPCODE: u16 = 12;
const BIS_OPCODE: u16 = 13;
const XOR_OPCODE: u16 = 14;
const AND_OPCODE: u16 = 15;

// MSP430X instruction constants
const MOVA_OPCODE: u16 = 0;
const CMPA_OPCODE: u16 = 1;
const ADDA_OPCODE: u16 = 2;
const SUBA_OPCODE: u16 = 3;
const RRCM_OPCODE: u16 = 4;
const RRAM_OPCODE: u16 = 5;
const RLAM_OPCODE: u16 = 6;
const RRUM_OPCODE: u16 = 7;

const CALLA_BASE: u16 = 0x1340;
const PUSHM_BASE: u16 = 0x1400;
const POPM_BASE: u16 = 0x1600;
const RETA_OPCODE: u16 = 0x0110;

// Bit masks
const SINGLE_OPERAND_REGISTER_MASK: u16 = 0b1111;
const SINGLE_OPERAND_OPCODE_MASK: u16 = 0b0000_0011_1000_0000;
const SINGLE_OPERAND_SOURCE_MASK: u16 = 0b11_0000;
const SINGLE_OPERAND_WIDTH_MASK: u16 = 0b100_0000;
const INST_TYPE_MASK: u16 = 0b1110_0000_0000_0000;
const SINGLE_OPERAND_INSTRUCTION: u16 = 0b0000_0000_0000_0000;
const JMP_INSTRUCTION: u16 = 0b0010_0000_0000_0000;
const JMP_CONDITION_MASK: u16 = 0b0001_1100_0000_0000;
const JMP_OFFSET: u16 = 0b0000001111111111;
const TWO_OPERAND_OPCODE_MASK: u16 = 0b1111_0000_0000_0000;
const TWO_OPERAND_SOURCE_MASK: u16 = 0b1111_0000_0000;
const TWO_OPERAND_AD_MASK: u16 = 0b1000_0000;
const TWO_OPERAND_WIDTH: u16 = 0b100_0000;
const TWO_OPERAND_AS: u16 = 0b11_0000;
const TWO_OPERAND_DESTINATION: u16 = 0b1111;

pub type Result<T> = std::result::Result<T, DecodeError>;

/// Decode the next instruction from data
/// Returns the instruction and updates the instruction size
pub fn decode(data: &[u8]) -> Result<Instruction> {
    if data.len() < 2 {
        return Err(DecodeError::MissingInstruction);
    }

    let (int_bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
    let first_word = u16::from_le_bytes(int_bytes.try_into().unwrap());

    // Check for extension word (18xx - 1Fxx range)
    if (first_word & 0xF800) == 0x1800 {
        let (ext, remaining_after_ext) = ExtensionWord::parse(data)?;
        return decode_with_extension(&ext, remaining_after_ext);
    }

    // Check for MSP430X instructions without extension word
    if let Some(inst) = try_decode_msp430x(first_word, remaining_data) {
        return inst;
    }

    // Decode base MSP430 instructions
    decode_base_msp430(first_word, remaining_data)
}

/// Try to decode MSP430X instructions that don't require extension words
fn try_decode_msp430x(first_word: u16, _data: &[u8]) -> Option<Result<Instruction>> {
    // RETA: 0x0110
    if first_word == RETA_OPCODE {
        return Some(Ok(Instruction::Reta(Reta::new())));
    }

    None
}

/// Decode instruction with extension word
fn decode_with_extension(ext: &ExtensionWord, data: &[u8]) -> Result<Instruction> {
    if data.len() < 2 {
        return Err(DecodeError::MissingInstruction);
    }

    let (word_bytes, remaining) = data.split_at(2);
    let inst_word = u16::from_le_bytes([word_bytes[0], word_bytes[1]]);

    // MSP430X address instructions (0xxx range with extension)
    let opcode = (inst_word >> 4) & 0xF;

    // MOVA, CMPA, ADDA, SUBA basic decoding
    // Format varies by addressing mode - this is simplified
    match opcode {
        MOVA_OPCODE => {
            // Simplified MOVA decoding - register to register
            let src_reg = ((inst_word >> 8) & 0xF) as u8;
            let dst_reg = (inst_word & 0xF) as u8;

            let source = operand::Operand::RegisterDirect(src_reg);
            let dest = operand::Operand::RegisterDirect(dst_reg);

            Ok(Instruction::Mova(AddressInstruction::new("mova", source, dest)))
        }
        _ => Err(DecodeError::InvalidOpcode(opcode)),
    }
}

/// Decode base MSP430 instruction (without extension word)
fn decode_base_msp430(first_word: u16, remaining_data: &[u8]) -> Result<Instruction> {
    let inst_type = first_word & INST_TYPE_MASK;

    match inst_type {
        SINGLE_OPERAND_INSTRUCTION => {
            decode_single_operand(first_word, remaining_data)
        }
        JMP_INSTRUCTION => {
            decode_jxx(first_word)
        }
        _ => {
            decode_two_operand(first_word, remaining_data)
        }
    }
}

fn decode_single_operand(first_word: u16, remaining_data: &[u8]) -> Result<Instruction> {
    let opcode = (SINGLE_OPERAND_OPCODE_MASK & first_word) >> 7;
    let register = (SINGLE_OPERAND_REGISTER_MASK & first_word) as u8;
    let source_addressing = (SINGLE_OPERAND_SOURCE_MASK & first_word) >> 4;
    let operand_width =
        OperandWidth::from(((SINGLE_OPERAND_WIDTH_MASK & first_word) >> 6) as u8);

    let (source, _) = parse_source(register, source_addressing, remaining_data, None)?;

    match opcode {
        RRC_OPCODE => Ok(Instruction::Rrc(Rrc::new(source, Some(operand_width)))),
        SWPB_OPCODE => Ok(Instruction::Swpb(Swpb::new(source, None))),
        RRA_OPCODE => Ok(Instruction::Rra(Rra::new(source, Some(operand_width)))),
        SXT_OPCODE => Ok(Instruction::Sxt(Sxt::new(source, None))),
        PUSH_OPCODE => Ok(Instruction::Push(Push::new(source, Some(operand_width)))),
        CALL_OPCODE => Ok(Instruction::Call(Call::new(source, None))),
        RETI_OPCODE => Ok(Instruction::Reti(Reti::new())),
        _ => Err(DecodeError::InvalidOpcode(opcode)),
    }
}

fn decode_jxx(first_word: u16) -> Result<Instruction> {
    let condition = (first_word & JMP_CONDITION_MASK) >> 10;
    let offset = jxx_fix_offset(first_word & JMP_OFFSET);

    match condition {
        0 => Ok(Instruction::Jnz(Jnz::new(offset))),
        1 => Ok(Instruction::Jz(Jz::new(offset))),
        2 => Ok(Instruction::Jlo(Jlo::new(offset))),
        3 => Ok(Instruction::Jc(Jc::new(offset))),
        4 => Ok(Instruction::Jn(Jn::new(offset))),
        5 => Ok(Instruction::Jge(Jge::new(offset))),
        6 => Ok(Instruction::Jl(Jl::new(offset))),
        7 => Ok(Instruction::Jmp(Jmp::new(offset))),
        _ => Err(DecodeError::InvalidJumpCondition(condition)),
    }
}

fn decode_two_operand(first_word: u16, remaining_data: &[u8]) -> Result<Instruction> {
    let opcode = (first_word & TWO_OPERAND_OPCODE_MASK) >> 12;
    let source_register = ((first_word & TWO_OPERAND_SOURCE_MASK) >> 8) as u8;
    let ad = (first_word & TWO_OPERAND_AD_MASK) >> 7;
    let operand_width = OperandWidth::from(((first_word & TWO_OPERAND_WIDTH) >> 6) as u8);
    let source_addressing = (first_word & TWO_OPERAND_AS) >> 4;
    let destination_register = (first_word & TWO_OPERAND_DESTINATION) as u8;

    let (source, remaining_data) =
        parse_source(source_register, source_addressing, remaining_data, None)?;

    let destination = parse_destination(destination_register, ad, remaining_data, None)?;

    match opcode {
        MOV_OPCODE => {
            let inst = Mov::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Mov(inst)),
            }
        }
        ADD_OPCODE => {
            let inst = Add::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Add(inst)),
            }
        }
        ADDC_OPCODE => {
            let inst = Addc::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Addc(inst)),
            }
        }
        SUBC_OPCODE => {
            let inst = Subc::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Subc(inst)),
            }
        }
        SUB_OPCODE => {
            let inst = Sub::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Sub(inst)),
            }
        }
        CMP_OPCODE => {
            let inst = Cmp::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Cmp(inst)),
            }
        }
        DADD_OPCODE => {
            let inst = Dadd::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Dadd(inst)),
            }
        }
        BIT_OPCODE => Ok(Instruction::Bit(Bit::new(
            source,
            operand_width,
            destination,
        ))),
        BIC_OPCODE => {
            let inst = Bic::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Bic(inst)),
            }
        }
        BIS_OPCODE => {
            let inst = Bis::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Bis(inst)),
            }
        }
        XOR_OPCODE => {
            let inst = Xor::new(source, operand_width, destination);
            match inst.emulate() {
                Some(inst) => Ok(inst),
                None => Ok(Instruction::Xor(inst)),
            }
        }
        AND_OPCODE => Ok(Instruction::And(And::new(
            source,
            operand_width,
            destination,
        ))),
        _ => Err(DecodeError::InvalidOpcode(opcode)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use operand::Operand;

    #[test]
    fn base_msp430_jmp() {
        let data = [0x00, 0x3c];
        let inst = decode(&data);
        assert_eq!(inst, Ok(Instruction::Jmp(Jmp::new(0))));
    }

    #[test]
    fn base_msp430_mov() {
        let data = [0x4f, 0x4f]; // mov.w r15, r15
        let inst = decode(&data);
        assert!(inst.is_ok());
    }

    #[test]
    fn msp430x_reta() {
        let data = [0x10, 0x01]; // RETA opcode
        let inst = decode(&data);
        assert_eq!(inst, Ok(Instruction::Reta(Reta::new())));
    }

    #[test]
    fn empty_data() {
        let data = [];
        assert_eq!(decode(&data), Err(DecodeError::MissingInstruction));
    }
}
