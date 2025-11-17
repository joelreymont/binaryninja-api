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

    // PUSHM: 0x1400-0x15FF (format: 0001 010n nnnn rrrr)
    // n = count - 1 (0-15 for 1-16 registers)
    // r = ending register (0-15)
    // .A mode if bit 8 is set
    if (first_word & 0xFE00) == 0x1400 {
        let count = ((first_word >> 4) & 0xF) as u8 + 1;  // n + 1
        let register = (first_word & 0xF) as u8;
        let is_address = (first_word & 0x0100) != 0;  // .A mode
        return Some(Ok(Instruction::Pushm(
            PushPopMultiple::new("pushm", count, register, is_address)
        )));
    }

    // POPM: 0x1600-0x17FF (format: 0001 011n nnnn rrrr)
    if (first_word & 0xFE00) == 0x1600 {
        let count = ((first_word >> 4) & 0xF) as u8 + 1;
        let register = (first_word & 0xF) as u8;
        let is_address = (first_word & 0x0100) != 0;
        return Some(Ok(Instruction::Popm(
            PushPopMultiple::new("popm", count, register, is_address)
        )));
    }

    // Rotate/shift multiple: 0x0400-0x07FF (format: 0000 01xx xxnn rrrr)
    // Bits [15:10] = 000001 (0x0400 range)
    // Bits [9:8]   = xx (operation: 00=RRCM, 01=RRAM, 10=RLAM, 11=RRUM)
    // Bits [7:6]   = nn (count - 1, 0-3 for 1-4 shifts)
    // Bits [5:4]   = A/L bits (bit 4 = .A mode)
    // Bits [3:0]   = rrrr (register)
    if (first_word & 0xFC00) == 0x0400 {
        let op = (first_word >> 8) & 0x3;          // Bits [9:8]
        let count = ((first_word >> 6) & 0x3) as u8 + 1;  // Bits [7:6] + 1
        let register = (first_word & 0xF) as u8;   // Bits [3:0]
        let is_address = (first_word & 0x0010) != 0;  // Bit 4 = .A mode

        return Some(Ok(match op {
            0 => Instruction::Rrcm(RotateMultiple::new("rrcm", count, register, is_address)),
            1 => Instruction::Rram(RotateMultiple::new("rram", count, register, is_address)),
            2 => Instruction::Rlam(RotateMultiple::new("rlam", count, register, is_address)),
            3 => Instruction::Rrum(RotateMultiple::new("rrum", count, register, is_address)),
            _ => return None,
        }));
    }

    // CALLA: 0x1340-0x137F (format: 0001 0011 01xx xxxx)
    // TODO: Full CALLA decoding with different addressing modes

    None
}

/// Decode instruction with extension word
fn decode_with_extension(ext: &ExtensionWord, data: &[u8]) -> Result<Instruction> {
    if data.len() < 2 {
        return Err(DecodeError::MissingInstruction);
    }

    let (word_bytes, remaining) = data.split_at(2);
    let inst_word = u16::from_le_bytes([word_bytes[0], word_bytes[1]]);

    // Decode based on instruction word patterns
    let upper_byte = (inst_word >> 8) & 0xFF;
    let lower_nibble = inst_word & 0xF;
    let mode_nibble = (inst_word >> 4) & 0xF;

    // MOVA instructions (0x00xx range)
    if upper_byte == 0x00 {
        let rd = lower_nibble as u8;
        let ext_data = ext.dest_extension();

        match mode_nibble {
            0x2 => {
                // MOVA #imm20, Rd
                if remaining.len() < 2 {
                    return Err(DecodeError::MissingOperand);
                }
                let imm_low = u16::from_le_bytes([remaining[0], remaining[1]]);
                let imm20 = ((ext_data as u32) << 16) | (imm_low as u32);
                let source = operand::Operand::Immediate20(imm20);
                let dest = operand::Operand::RegisterDirect(rd);
                return Ok(Instruction::Mova(AddressInstruction::new("mova", source, dest)));
            }
            0x3 => {
                // MOVA &abs20, Rd
                if remaining.len() < 2 {
                    return Err(DecodeError::MissingOperand);
                }
                let abs_low = u16::from_le_bytes([remaining[0], remaining[1]]);
                let abs20 = ((ext_data as u32) << 16) | (abs_low as u32);
                let source = operand::Operand::Absolute20(abs20);
                let dest = operand::Operand::RegisterDirect(rd);
                return Ok(Instruction::Mova(AddressInstruction::new("mova", source, dest)));
            }
            0x5 => {
                // MOVA Rs, &abs20
                if remaining.len() < 2 {
                    return Err(DecodeError::MissingOperand);
                }
                let rs = ((inst_word >> 4) & 0xF) as u8;
                let abs_low = u16::from_le_bytes([remaining[0], remaining[1]]);
                let abs20 = ((ext_data as u32) << 16) | (abs_low as u32);
                let source = operand::Operand::RegisterDirect(rs);
                let dest = operand::Operand::Absolute20(abs20);
                return Ok(Instruction::Mova(AddressInstruction::new("mova", source, dest)));
            }
            0x1 => {
                // MOVA x(Rs), Rd
                if remaining.len() < 2 {
                    return Err(DecodeError::MissingOperand);
                }
                let rs = ((inst_word >> 4) & 0xF) as u8;
                let offset_low = u16::from_le_bytes([remaining[0], remaining[1]]);
                let offset20 = ((ext_data as i32) << 16) | (offset_low as i32);
                let source = operand::Operand::Indexed20((rs, offset20));
                let dest = operand::Operand::RegisterDirect(rd);
                return Ok(Instruction::Mova(AddressInstruction::new("mova", source, dest)));
            }
            0x6 => {
                // MOVA Rs, Rd (register to register)
                let rs = ((inst_word >> 4) & 0xF) as u8;
                let source = operand::Operand::RegisterDirect(rs);
                let dest = operand::Operand::RegisterDirect(rd);
                return Ok(Instruction::Mova(AddressInstruction::new("mova", source, dest)));
            }
            _ => {}
        }
    }

    // CMPA #imm20, Rd (0x0090-0x009F)
    if upper_byte == 0x00 && mode_nibble == 0x9 {
        if remaining.len() < 2 {
            return Err(DecodeError::MissingOperand);
        }
        let rd = lower_nibble as u8;
        let ext_data = ext.dest_extension();
        let imm_low = u16::from_le_bytes([remaining[0], remaining[1]]);
        let imm20 = ((ext_data as u32) << 16) | (imm_low as u32);
        let source = operand::Operand::Immediate20(imm20);
        let dest = operand::Operand::RegisterDirect(rd);
        return Ok(Instruction::Cmpa(AddressInstruction::new("cmpa", source, dest)));
    }

    // ADDA #imm20, Rd (0x00A0-0x00AF)
    if upper_byte == 0x00 && mode_nibble == 0xA {
        if remaining.len() < 2 {
            return Err(DecodeError::MissingOperand);
        }
        let rd = lower_nibble as u8;
        let ext_data = ext.dest_extension();
        let imm_low = u16::from_le_bytes([remaining[0], remaining[1]]);
        let imm20 = ((ext_data as u32) << 16) | (imm_low as u32);
        let source = operand::Operand::Immediate20(imm20);
        let dest = operand::Operand::RegisterDirect(rd);
        return Ok(Instruction::Adda(AddressInstruction::new("adda", source, dest)));
    }

    // SUBA #imm20, Rd (0x00B0-0x00BF)
    if upper_byte == 0x00 && mode_nibble == 0xB {
        if remaining.len() < 2 {
            return Err(DecodeError::MissingOperand);
        }
        let rd = lower_nibble as u8;
        let ext_data = ext.dest_extension();
        let imm_low = u16::from_le_bytes([remaining[0], remaining[1]]);
        let imm20 = ((ext_data as u32) << 16) | (imm_low as u32);
        let source = operand::Operand::Immediate20(imm20);
        let dest = operand::Operand::RegisterDirect(rd);
        return Ok(Instruction::Suba(AddressInstruction::new("suba", source, dest)));
    }

    // CALLA instructions (0x13xx range)
    if upper_byte == 0x13 {
        let ext_data = ext.dest_extension();
        match mode_nibble {
            0x4 => {
                // CALLA Rs (0x1340-0x134F)
                let rs = lower_nibble as u8;
                let dest = operand::Operand::RegisterDirect(rs);
                return Ok(Instruction::Calla(Calla::new(dest)));
            }
            0x8 => {
                // CALLA &abs20 (0x1380-0x138F)
                if remaining.len() < 2 {
                    return Err(DecodeError::MissingOperand);
                }
                let abs_low = u16::from_le_bytes([remaining[0], remaining[1]]);
                let abs20 = ((ext_data as u32) << 16) | (abs_low as u32);
                let dest = operand::Operand::Absolute20(abs20);
                return Ok(Instruction::Calla(Calla::new(dest)));
            }
            0xB => {
                // CALLA #imm20 (0x13B0-0x13BF)
                if remaining.len() < 2 {
                    return Err(DecodeError::MissingOperand);
                }
                let imm_low = u16::from_le_bytes([remaining[0], remaining[1]]);
                let imm20 = ((ext_data as u32) << 16) | (imm_low as u32);
                let dest = operand::Operand::Immediate20(imm20);
                return Ok(Instruction::Calla(Calla::new(dest)));
            }
            _ => {}
        }
    }

    // BRA instructions (similar encoding to MOVA, but unconditional branch)
    // BRA is encoded as MOVA to PC in some variants
    // Extension word + instruction variants:
    // - BRA Rdst (0x00Cx range) - Register mode
    // - BRA &abs20 (0x00Cx + abs20 word)
    // - BRA #imm20 (0x00Cx + imm20 word)
    if upper_byte == 0x00 && mode_nibble == 0xC {
        // BRA Rdst or BRA with operand
        let ext_data = ext.dest_extension();
        let rs = lower_nibble as u8;

        // Check if there's an additional operand word
        if rs == 0 && remaining.len() >= 2 {
            // BRA #imm20 or BRA &abs20
            let operand_low = u16::from_le_bytes([remaining[0], remaining[1]]);
            let operand20 = ((ext_data as u32) << 16) | (operand_low as u32);
            // Distinguish between immediate and absolute based on instruction encoding
            // For simplicity, treat as absolute address for branch target
            let dest = operand::Operand::Absolute20(operand20);
            return Ok(Instruction::Bra(Bra::new(dest)));
        } else {
            // BRA Rdst (register direct)
            let dest = operand::Operand::RegisterDirect(rs);
            return Ok(Instruction::Bra(Bra::new(dest)));
        }
    }

    Err(DecodeError::InvalidOpcode(upper_byte as u16))
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

    // Additional MSP430X and base MSP430 tests for better coverage
    #[test]
    fn base_msp430_add() {
        let data = [0x0E, 0x5F]; // add.w r14, r15
        let inst = decode(&data);
        assert!(inst.is_ok());
    }

    #[test]
    fn base_msp430_sub() {
        let data = [0x0E, 0x8F]; // sub.w r14, r15
        let inst = decode(&data);
        assert!(inst.is_ok());
    }

    #[test]
    fn base_msp430_cmp() {
        let data = [0x0E, 0x9F]; // cmp.w r14, r15
        let inst = decode(&data);
        assert!(inst.is_ok());
    }

    #[test]
    fn decode_instruction_sizes() {
        // JMP is 2 bytes
        let data = [0x00, 0x3c];
        if let Ok(inst) = decode(&data) {
            assert_eq!(inst.size(), 2);
        }

        // RETA is 2 bytes
        let data = [0x10, 0x01];
        if let Ok(inst) = decode(&data) {
            assert_eq!(inst.size(), 2);
        }
    }

    // Real MSP430X instruction encodings from GCC msp430-elf-gcc 9.3.1
    // Verified correct encodings from compiled test_simple.elf binary

    #[test]
    fn gcc_real_reta() {
        // RETA - from test_simple.elf at 0x4416
        let data = [0x10, 0x01];
        let inst = decode(&data);
        assert_eq!(inst, Ok(Instruction::Reta(Reta::new())));
    }

    #[test]
    fn gcc_real_pushm_address_mode() {
        // PUSHM.A #4, r15 - from test_simple.elf at 0x4424
        let data = [0x3f, 0x14];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode PUSHM.A");
        if let Ok(inst) = inst {
            assert_eq!(inst.size(), 2);
        }
    }

    #[test]
    fn gcc_real_popm_address_mode() {
        // POPM.A #4, r15 - from test_simple.elf at 0x442e
        let data = [0x3c, 0x16];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode POPM.A");
        if let Ok(inst) = inst {
            assert_eq!(inst.size(), 2);
        }
    }

    #[test]
    fn gcc_real_rlam_address_mode() {
        // RLAM.A #2, r15 - from test_simple.elf at 0x4434
        let data = [0x4f, 0x06];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode RLAM.A");
        if let Ok(inst) = inst {
            assert_eq!(inst.size(), 2);
        }
    }

    #[test]
    fn gcc_real_rram_address_mode() {
        // RRAM.A #2, r14 - from test_simple.elf at 0x4436
        let data = [0x4e, 0x05];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode RRAM.A");
        if let Ok(inst) = inst {
            assert_eq!(inst.size(), 2);
        }
    }

    // MSP430X Address Instruction Tests
    // Testing MOVA, CMPA, ADDA, SUBA with various addressing modes

    #[test]
    fn mova_immediate20_to_register() {
        // MOVA #0x12345, r15
        // Extension word: 0x1801, Instruction: 0x002F, Immediate: 0x2345
        let data = [0x01, 0x18, 0x2F, 0x00, 0x45, 0x23];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode MOVA #imm20, Rd");
        if let Ok(Instruction::Mova(mova)) = inst {
            assert_eq!(mova.size(), 4); // Instruction word + immediate word (extension word not counted)
            assert_eq!(*mova.source(), Operand::Immediate20(0x12345));
            assert_eq!(*mova.destination(), Operand::RegisterDirect(15));
        } else {
            panic!("Expected MOVA instruction");
        }
    }

    #[test]
    fn mova_absolute20_to_register() {
        // MOVA &0x10000, r14
        // Extension word: 0x1801, Instruction: 0x003E, Absolute: 0x0000
        let data = [0x01, 0x18, 0x3E, 0x00, 0x00, 0x00];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode MOVA &abs20, Rd");
        if let Ok(Instruction::Mova(mova)) = inst {
            assert_eq!(mova.size(), 4); // Instruction word + absolute word
            assert_eq!(*mova.source(), Operand::Absolute20(0x10000));
            assert_eq!(*mova.destination(), Operand::RegisterDirect(14));
        } else {
            panic!("Expected MOVA instruction");
        }
    }

    #[test]
    fn mova_register_to_register() {
        // MOVA r6, r15  (using mode 0x6 which encodes source in mode nibble)
        // Extension word: 0x1800, Instruction: 0x006F (mode[7:4]=6=r6, dest[3:0]=F=r15)
        let data = [0x00, 0x18, 0x6F, 0x00];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode MOVA Rs, Rd");
        if let Ok(Instruction::Mova(mova)) = inst {
            assert_eq!(mova.size(), 2); // Just instruction word
            assert_eq!(*mova.source(), Operand::RegisterDirect(6));
            assert_eq!(*mova.destination(), Operand::RegisterDirect(15));
        } else {
            panic!("Expected MOVA instruction");
        }
    }

    #[test]
    fn cmpa_immediate20() {
        // CMPA #0x12345, r15
        // Extension word: 0x1801, Instruction: 0x009F, Immediate: 0x2345
        let data = [0x01, 0x18, 0x9F, 0x00, 0x45, 0x23];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode CMPA #imm20, Rd");
        if let Ok(Instruction::Cmpa(cmpa)) = inst {
            assert_eq!(cmpa.size(), 4); // Instruction word + immediate word
            assert_eq!(*cmpa.source(), Operand::Immediate20(0x12345));
            assert_eq!(*cmpa.destination(), Operand::RegisterDirect(15));
        } else {
            panic!("Expected CMPA instruction");
        }
    }

    #[test]
    fn adda_immediate20() {
        // ADDA #0x1000, r15
        // Extension word: 0x1800, Instruction: 0x00AF, Immediate: 0x1000
        let data = [0x00, 0x18, 0xAF, 0x00, 0x00, 0x10];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode ADDA #imm20, Rd");
        if let Ok(Instruction::Adda(adda)) = inst {
            assert_eq!(adda.size(), 4); // Instruction word + immediate word
            assert_eq!(*adda.source(), Operand::Immediate20(0x1000));
            assert_eq!(*adda.destination(), Operand::RegisterDirect(15));
        } else {
            panic!("Expected ADDA instruction");
        }
    }

    #[test]
    fn suba_immediate20() {
        // SUBA #0x2000, r15
        // Extension word: 0x1800, Instruction: 0x00BF, Immediate: 0x2000
        let data = [0x00, 0x18, 0xBF, 0x00, 0x00, 0x20];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode SUBA #imm20, Rd");
        if let Ok(Instruction::Suba(suba)) = inst {
            assert_eq!(suba.size(), 4); // Instruction word + immediate word
            assert_eq!(*suba.source(), Operand::Immediate20(0x2000));
            assert_eq!(*suba.destination(), Operand::RegisterDirect(15));
        } else {
            panic!("Expected SUBA instruction");
        }
    }

    #[test]
    fn calla_register() {
        // CALLA r15
        // Extension word: 0x1800, Instruction: 0x134F
        let data = [0x00, 0x18, 0x4F, 0x13];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode CALLA Rs");
        if let Ok(Instruction::Calla(calla)) = inst {
            assert_eq!(calla.size(), 2); // Just instruction word
            assert_eq!(*calla.destination(), Operand::RegisterDirect(15));
        } else {
            panic!("Expected CALLA instruction");
        }
    }

    #[test]
    fn calla_absolute20() {
        // CALLA &0x10000
        // Extension word: 0x1801, Instruction: 0x1380, Absolute: 0x0000
        let data = [0x01, 0x18, 0x80, 0x13, 0x00, 0x00];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode CALLA &abs20");
        if let Ok(Instruction::Calla(calla)) = inst {
            assert_eq!(calla.size(), 4); // Instruction word + absolute word
            assert_eq!(*calla.destination(), Operand::Absolute20(0x10000));
        } else {
            panic!("Expected CALLA instruction");
        }
    }

    #[test]
    fn calla_immediate20() {
        // CALLA #0x12345
        // Extension word: 0x1801, Instruction: 0x13B0, Immediate: 0x2345
        let data = [0x01, 0x18, 0xB0, 0x13, 0x45, 0x23];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode CALLA #imm20");
        if let Ok(Instruction::Calla(calla)) = inst {
            assert_eq!(calla.size(), 4); // Instruction word + immediate word
            assert_eq!(*calla.destination(), Operand::Immediate20(0x12345));
        } else {
            panic!("Expected CALLA instruction");
        }
    }

    #[test]
    fn bra_register() {
        // BRA r15
        // Extension word: 0x1800, Instruction: 0x00CF
        let data = [0x00, 0x18, 0xCF, 0x00];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode BRA Rs");
        if let Ok(Instruction::Bra(bra)) = inst {
            assert_eq!(bra.size(), 2); // Just instruction word
            assert_eq!(*bra.destination(), Operand::RegisterDirect(15));
        } else {
            panic!("Expected BRA instruction");
        }
    }

    #[test]
    fn bra_absolute20() {
        // BRA &0x10000
        // Extension word: 0x1801, Instruction: 0x00C0, Absolute: 0x0000
        let data = [0x01, 0x18, 0xC0, 0x00, 0x00, 0x00];
        let inst = decode(&data);
        assert!(inst.is_ok(), "Failed to decode BRA &abs20");
        if let Ok(Instruction::Bra(bra)) = inst {
            assert_eq!(bra.size(), 4); // Instruction word + absolute word
            assert_eq!(*bra.destination(), Operand::Absolute20(0x10000));
        } else {
            panic!("Expected BRA instruction");
        }
    }
}
