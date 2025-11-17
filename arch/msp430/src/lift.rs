use crate::architecture::offset_to_absolute;
use crate::flag::{Flag, FlagWrite};
use crate::register::Register;

use binaryninja::{architecture::FlagCondition, low_level_il::lifting::LowLevelILLabel};

use msp430_asm_extended::emulate::Emulated;
use msp430_asm_extended::instruction::Instruction;
use msp430_asm_extended::jxx::Jxx;
use msp430_asm_extended::operand::{Operand, OperandWidth};
use msp430_asm_extended::single_operand::SingleOperand;
use msp430_asm_extended::two_operand::TwoOperand;

use binaryninja::low_level_il::expression::ValueExpr;
use binaryninja::low_level_il::{LowLevelILMutableExpression, LowLevelILMutableFunction};
use log::info;

macro_rules! auto_increment {
    ($src:expr, $il:ident) => {
        if let Operand::RegisterIndirectAutoIncrement(r) = $src {
            $il.set_reg(
                2,
                Register::try_from(*r as u32).unwrap(),
                $il.add(
                    2,
                    $il.reg(2, Register::try_from(*r as u32).unwrap()),
                    $il.const_int(2, 2),
                ),
            )
            .append();
        }
    };
}

macro_rules! one_operand {
    ($source:expr, $il:ident, $op:ident) => {
        match $source {
            Operand::RegisterDirect(r) => $il
                .set_reg(2, Register::try_from(*r as u32).unwrap(), $op)
                .append(),
            Operand::Indexed((r, offset)) => $il
                .store(
                    2,
                    $il.add(
                        2,
                        $il.reg(2, Register::try_from(*r as u32).unwrap()),
                        $il.const_int(2, *offset as u64),
                    ),
                    $op,
                )
                .append(),
            Operand::Indexed20((r, offset)) => $il
                .store(
                    3,
                    $il.add(
                        3,
                        $il.reg(3, Register::try_from(*r as u32).unwrap()),
                        $il.const_int(3, *offset as u64),
                    ),
                    $op,
                )
                .append(),
            Operand::Symbolic(offset) => $il
                .store(2, $il.add(2, $il.reg(2, Register::Pc), *offset as u64), $op)
                .append(),
            Operand::Symbolic20(offset) => $il
                .store(3, $il.add(3, $il.reg(3, Register::Pc), *offset as u64), $op)
                .append(),
            Operand::Absolute(val) => $il.store(2, $il.const_ptr(*val as u64), $op).append(),
            Operand::Absolute20(val) => $il.store(3, $il.const_ptr(*val as u64), $op).append(),
            Operand::Immediate(_) => $op.append(),
            Operand::Immediate20(_) => $op.append(),
            Operand::RegisterIndirect(r) => $il
                .store(2, $il.reg(2, Register::try_from(*r as u32).unwrap()), $op)
                .append(),
            Operand::RegisterIndirectAutoIncrement(r) => {
                $il.store(2, $il.reg(2, Register::try_from(*r as u32).unwrap()), $op)
                    .append();
                $il.set_reg(
                    2,
                    Register::try_from(*r as u32).unwrap(),
                    $il.add(
                        2,
                        $il.reg(2, Register::try_from(*r as u32).unwrap()),
                        $il.const_int(2, 2),
                    ),
                )
                .append()
            }
            Operand::Constant(val) => $il.store(2, $il.const_int(2, *val as u64), $op).append(),
        };
    };
}

macro_rules! two_operand {
    ($destination:expr, $il:ident, $op:ident) => {
        match $destination {
            Operand::RegisterDirect(r) => $il
                .set_reg(2, Register::try_from(*r as u32).unwrap(), $op)
                .append(),
            Operand::Indexed((r, offset)) => $il
                .store(
                    2,
                    $il.add(
                        2,
                        $il.reg(2, Register::try_from(*r as u32).unwrap()),
                        $il.const_int(2, *offset as u64),
                    ),
                    $op,
                )
                .append(),
            Operand::Indexed20((r, offset)) => $il
                .store(
                    3,
                    $il.add(
                        3,
                        $il.reg(3, Register::try_from(*r as u32).unwrap()),
                        $il.const_int(3, *offset as u64),
                    ),
                    $op,
                )
                .append(),
            Operand::Symbolic(offset) => $il
                .store(2, $il.add(2, $il.reg(2, Register::Pc), *offset as u64), $op)
                .append(),
            Operand::Symbolic20(offset) => $il
                .store(3, $il.add(3, $il.reg(3, Register::Pc), *offset as u64), $op)
                .append(),
            Operand::Absolute(val) => $il.store(2, $il.const_ptr(*val as u64), $op).append(),
            Operand::Absolute20(val) => $il.store(3, $il.const_ptr(*val as u64), $op).append(),
            _ => {
                unreachable!()
            }
        };
    };
}

macro_rules! emulated {
    ($inst:ident, $il:ident, $op:ident) => {
        match $inst.destination() {
            Some(Operand::RegisterDirect(r)) => $il
                .set_reg(2, Register::try_from(*r as u32).unwrap(), $op)
                .append(),
            Some(Operand::Indexed((r, offset))) => $il
                .store(
                    2,
                    $il.add(
                        2,
                        $il.reg(2, Register::try_from(*r as u32).unwrap()),
                        $il.const_int(2, *offset as u64),
                    ),
                    $op,
                )
                .append(),
            Some(Operand::Indexed20((r, offset))) => $il
                .store(
                    3,
                    $il.add(
                        3,
                        $il.reg(3, Register::try_from(*r as u32).unwrap()),
                        $il.const_int(3, *offset as u64),
                    ),
                    $op,
                )
                .append(),
            Some(Operand::Symbolic(offset)) => $il
                .store(2, $il.add(2, $il.reg(2, Register::Pc), *offset as u64), $op)
                .append(),
            Some(Operand::Symbolic20(offset)) => $il
                .store(3, $il.add(3, $il.reg(3, Register::Pc), *offset as u64), $op)
                .append(),
            Some(Operand::Absolute(val)) => $il.store(2, $il.const_ptr(*val as u64), $op).append(),
            Some(Operand::Absolute20(val)) => $il.store(3, $il.const_ptr(*val as u64), $op).append(),
            _ => {
                unreachable!()
            }
        };
    };
}

macro_rules! conditional_jump {
    ($addr:ident, $inst:ident, $cond:ident, $il:ident) => {
        let true_addr = offset_to_absolute($addr, $inst.offset());
        let false_addr = $addr + $inst.size() as u64;

        // Try to get existing labels for both branches
        let true_label_opt = $il.label_for_address(true_addr);
        let false_label_opt = $il.label_for_address(false_addr);

        if let Some(mut true_label) = true_label_opt {
            if let Some(mut false_label) = false_label_opt {
                // Both labels exist - simple conditional
                $il.if_expr($cond, &mut true_label, &mut false_label).append();
            } else {
                // True label exists, create false label for fall-through
                let mut false_label = LowLevelILLabel::new();
                $il.if_expr($cond, &mut true_label, &mut false_label).append();
                $il.mark_label(&mut false_label);
            }
        } else if let Some(mut false_label) = false_label_opt {
            // False label exists, create true label for jump
            let mut true_label = LowLevelILLabel::new();
            $il.if_expr($cond, &mut true_label, &mut false_label).append();
            $il.mark_label(&mut true_label);
            $il.jump($il.const_ptr(true_addr)).append();
        } else {
            // Neither label exists - create both
            let mut true_label = LowLevelILLabel::new();
            let mut false_label = LowLevelILLabel::new();
            $il.if_expr($cond, &mut true_label, &mut false_label).append();
            $il.mark_label(&mut true_label);
            $il.jump($il.const_ptr(true_addr)).append();
            $il.mark_label(&mut false_label);
        }
    };
}

pub(crate) fn lift_instruction(inst: &Instruction, addr: u64, il: &LowLevelILMutableFunction) {
    match inst {
        Instruction::Rrc(inst) => {
            // Check if operand is PC - if so, treat as unimplemented to avoid breaking control flow
            if let Operand::RegisterDirect(0) = inst.source() {
                // RRC PC is unusual and would break control flow analysis
                il.unimplemented().append();
            } else {
                let size = match inst.operand_width() {
                    Some(width) => width_to_size(width),
                    None => 2,
                };
                let src = il.const_int(size, 1);
                let dest = lift_source_operand(inst.source(), size, il);
                let op = match inst.operand_width() {
                    Some(OperandWidth::Byte) => {
                        il.sx(2, il.rrc(size, dest, src).with_flag_write(FlagWrite::All))
                    }
                    Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                        il.rrc(size, dest, src).with_flag_write(FlagWrite::All)
                    }
                };
                one_operand!(inst.source(), il, op);
            }
        }
        Instruction::Swpb(inst) => {
            let src = lift_source_operand(inst.source(), 2, il);
            let op = il.rol(2, src, il.const_int(2, 8));
            one_operand!(inst.source(), il, op);
        }
        Instruction::Rra(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let src = il.const_int(size, 1);
            let dest = lift_source_operand(inst.source(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => {
                    il.sx(2, il.ror(size, dest, src).with_flag_write(FlagWrite::Cnz))
                }
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                    il.ror(size, dest, src).with_flag_write(FlagWrite::Cnz)
                }
            };
            one_operand!(inst.source(), il, op);
            il.set_flag(Flag::V, il.const_int(0, 0)).append();
        }
        Instruction::Sxt(inst) => {
            // source is always 1 byte and instruction is always 2 bytes for sxt because we're sign
            // extending the low byte into the high and the result is always 2 bytes
            let src = lift_source_operand(inst.source(), 1, il);
            let op = il.sx(2, src).with_flag_write(FlagWrite::Nz);
            one_operand!(inst.source(), il, op);
            il.set_flag(Flag::V, il.const_int(0, 0)).append();
            il.set_flag(Flag::C, il.not(0, il.flag(Flag::Z))).append();
        }
        Instruction::Push(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let src = lift_source_operand(inst.source(), size, il);
            il.push(size, src).append();
            auto_increment!(inst.source(), il);
        }
        Instruction::Call(inst) => {
            // TODO: verify the special autoincrement behavior Josh implemented?
            let src = if let Operand::Immediate(src) = inst.source() {
                il.const_ptr(*src as u64)
            } else {
                let size = match inst.operand_width() {
                    Some(width) => width_to_size(width),
                    None => 2,
                };

                lift_source_operand(inst.source(), size, il)
            };
            il.call(src).append();
            auto_increment!(inst.source(), il);
        }
        Instruction::Reti(_) => {
            il.set_reg(2, Register::Sr, il.pop(2))
                .with_flag_write(FlagWrite::All)
                .append();
            il.ret(il.pop(2)).append();
        }

        // Jxx instructions
        Instruction::Jnz(inst) => {
            let cond = il.flag_cond(FlagCondition::LLFC_NE);
            conditional_jump!(addr, inst, cond, il);
        }
        Instruction::Jz(inst) => {
            let cond = il.flag_cond(FlagCondition::LLFC_E);
            conditional_jump!(addr, inst, cond, il);
        }
        Instruction::Jlo(inst) => {
            let cond = il.flag_cond(FlagCondition::LLFC_ULT);
            conditional_jump!(addr, inst, cond, il);
        }
        Instruction::Jc(inst) => {
            let cond = il.flag_cond(FlagCondition::LLFC_UGE);
            conditional_jump!(addr, inst, cond, il);
        }
        Instruction::Jn(inst) => {
            let cond = il.flag_cond(FlagCondition::LLFC_NEG);
            conditional_jump!(addr, inst, cond, il);
        }
        Instruction::Jge(inst) => {
            let cond = il.flag_cond(FlagCondition::LLFC_SGE);
            conditional_jump!(addr, inst, cond, il);
        }
        Instruction::Jl(inst) => {
            let cond = il.flag_cond(FlagCondition::LLFC_SLT);
            conditional_jump!(addr, inst, cond, il);
        }
        Instruction::Jmp(inst) => {
            let fixed_addr = offset_to_absolute(addr, inst.offset());
            let label = il.label_for_address(fixed_addr);
            match label {
                Some(mut label) => {
                    il.goto(&mut label).append();
                }
                None => {
                    il.jump(il.const_ptr(fixed_addr)).append();
                }
            }
        }

        // two operand instructions
        Instruction::Mov(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = match inst.operand_width() {
                OperandWidth::Byte => il
                    .sx(2, lift_source_operand(inst.source(), size, il))
                    .build(),
                OperandWidth::Word | OperandWidth::Address => lift_source_operand(inst.source(), size, il),
            };
            two_operand!(inst.destination(), il, src);
            auto_increment!(inst.source(), il);
        }
        Instruction::Add(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let op = match inst.operand_width() {
                OperandWidth::Byte => {
                    il.sx(2, il.add(size, src, dest).with_flag_write(FlagWrite::All))
                }
                OperandWidth::Word | OperandWidth::Address => il.add(size, src, dest).with_flag_write(FlagWrite::All),
            };
            two_operand!(inst.destination(), il, op);
            auto_increment!(inst.source(), il);
        }
        Instruction::Addc(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let carry = il.flag(Flag::C);
            let op = match inst.operand_width() {
                OperandWidth::Byte => {
                    il.sx(2, il.adc(size, src, dest, carry).with_flag_write(FlagWrite::All))
                }
                OperandWidth::Word | OperandWidth::Address => {
                    il.adc(size, src, dest, carry).with_flag_write(FlagWrite::All)
                }
            };
            two_operand!(inst.destination(), il, op);
            auto_increment!(inst.source(), il);
        }
        Instruction::Subc(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let carry = il.flag(Flag::C);
            let op = match inst.operand_width() {
                OperandWidth::Byte => {
                    il.sx(2, il.sbb(size, dest, src, carry).with_flag_write(FlagWrite::All))
                }
                OperandWidth::Word | OperandWidth::Address => {
                    il.sbb(size, dest, src, carry).with_flag_write(FlagWrite::All)
                }
            };
            two_operand!(inst.destination(), il, op);
            auto_increment!(inst.source(), il);
        }
        Instruction::Sub(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let op = match inst.operand_width() {
                OperandWidth::Byte => {
                    il.sx(2, il.sub(size, src, dest).with_flag_write(FlagWrite::All))
                }
                OperandWidth::Word | OperandWidth::Address => il.sub(size, src, dest).with_flag_write(FlagWrite::All),
            };
            two_operand!(inst.destination(), il, op);
            auto_increment!(inst.source(), il);
        }
        Instruction::Cmp(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            il.sub(size, dest, src)
                .with_flag_write(FlagWrite::All)
                .append();
            auto_increment!(inst.source(), il);
        }
        Instruction::Dadd(inst) => {
            // DADD - Decimal (BCD) add
            // Note: LLIL doesn't have a native BCD operation, so we implement this
            // as regular binary addition. The decompilation won't be semantically
            // perfect for BCD operations, but control flow will be correct.
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let op = match inst.operand_width() {
                OperandWidth::Byte => {
                    il.sx(2, il.add(size, src, dest).with_flag_write(FlagWrite::All))
                }
                OperandWidth::Word | OperandWidth::Address => {
                    il.add(size, src, dest).with_flag_write(FlagWrite::All)
                }
            };
            two_operand!(inst.destination(), il, op);
            auto_increment!(inst.source(), il);
        }
        Instruction::Bit(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            il.and(size, src, dest)
                .with_flag_write(FlagWrite::Nz)
                .append();
            il.set_flag(Flag::V, il.const_int(0, 0)).append();
            il.set_flag(Flag::C, il.not(0, il.flag(Flag::Z))).append();
            auto_increment!(inst.source(), il);
        }
        Instruction::Bic(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let op = match inst.operand_width() {
                OperandWidth::Byte => il.sx(2, il.and(size, il.not(size, src), dest)),
                OperandWidth::Word | OperandWidth::Address => il.and(size, il.not(size, src), dest),
            };
            two_operand!(inst.destination(), il, op);
            auto_increment!(inst.source(), il);
        }
        Instruction::Bis(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let op = match inst.operand_width() {
                OperandWidth::Byte => il.sx(2, il.or(size, src, dest)),
                OperandWidth::Word | OperandWidth::Address => il.or(size, src, dest),
            };
            two_operand!(inst.destination(), il, op);
            auto_increment!(inst.source(), il);
        }
        Instruction::Xor(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let op = match inst.operand_width() {
                OperandWidth::Byte => {
                    il.sx(2, il.xor(size, src, dest).with_flag_write(FlagWrite::Nvz))
                }
                OperandWidth::Word | OperandWidth::Address => il.xor(size, src, dest).with_flag_write(FlagWrite::Nvz),
            };
            two_operand!(inst.destination(), il, op);
            il.set_flag(Flag::C, il.not(0, il.flag(Flag::Z))).append();
            auto_increment!(inst.source(), il);
        }
        Instruction::And(inst) => {
            let size = width_to_size(inst.operand_width());
            let src = lift_source_operand(inst.source(), size, il);
            let dest = lift_source_operand(inst.destination(), size, il);
            let op = match inst.operand_width() {
                OperandWidth::Byte => {
                    il.sx(2, il.and(size, src, dest).with_flag_write(FlagWrite::Nz))
                }
                OperandWidth::Word | OperandWidth::Address => il.and(size, src, dest).with_flag_write(FlagWrite::Nz),
            };
            two_operand!(inst.destination(), il, op);
            il.set_flag(Flag::V, il.const_int(0, 0)).append();
            il.set_flag(Flag::C, il.not(0, il.flag(Flag::Z))).append();
            auto_increment!(inst.source(), il);
        }

        // emulated
        Instruction::Adc(inst) => {
            // ADC dst is emulated as ADDC #0, dst
            // Effectively: dst = dst + carry
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let carry = il.flag(Flag::C);
            // ADC adds zero plus carry, which is just adding the carry
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => {
                    il.sx(2, il.adc(size, il.const_int(size, 0), dest, carry)
                        .with_flag_write(FlagWrite::All))
                }
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                    il.adc(size, il.const_int(size, 0), dest, carry)
                        .with_flag_write(FlagWrite::All)
                }
            };
            emulated!(inst, il, op);
        }
        Instruction::Br(inst) => {
            let dest = if let Some(Operand::Immediate(dest)) = inst.destination() {
                if let Some(mut label) = il.label_for_address(*dest as u64) {
                    il.goto(&mut label).append();
                    return;
                } else {
                    il.const_ptr(*dest as u64)
                }
            } else {
                lift_source_operand(&inst.destination().unwrap(), 2, il)
            };

            il.jump(dest).append();
        }
        Instruction::Clr(inst) => {
            let op = il.const_int(2, 0);
            emulated!(inst, il, op);
        }
        Instruction::Clrc(_) => {
            // TODO: should we lift clearing the C bit in the SR register as well?
            il.set_flag(Flag::C, il.const_int(0, 0)).append();
        }
        Instruction::Clrn(_) => {
            // TODO: should we lift clearing the N bit in the SR register as well?
            il.set_flag(Flag::N, il.const_int(0, 0)).append();
        }
        Instruction::Clrz(_) => {
            // TODO: should we lift clearing the Z bit in the SR register as well?
            il.set_flag(Flag::Z, il.const_int(0, 0)).append();
        }
        Instruction::Dadc(inst) => {
            // DADC - Decimal (BCD) add with carry (emulated as DADD #0, dst)
            // Note: LLIL doesn't have native BCD operations, so we implement this
            // as regular binary addition with carry. Control flow will be correct.
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let carry = il.flag(Flag::C);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => {
                    il.sx(2, il.adc(size, il.const_int(size, 0), dest, carry)
                        .with_flag_write(FlagWrite::All))
                }
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                    il.adc(size, il.const_int(size, 0), dest, carry)
                        .with_flag_write(FlagWrite::All)
                }
            };
            emulated!(inst, il, op);
        }
        Instruction::Dec(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => il.sx(
                    2,
                    il.sub(size, dest, il.const_int(size, 1))
                        .with_flag_write(FlagWrite::All),
                ),
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => il
                    .sub(size, dest, il.const_int(size, 1))
                    .with_flag_write(FlagWrite::All),
            };
            emulated!(inst, il, op);
        }
        Instruction::Decd(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => il.sx(
                    2,
                    il.sub(size, dest, il.const_int(size, 2))
                        .with_flag_write(FlagWrite::All),
                ),
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => il
                    .sub(size, dest, il.const_int(size, 2))
                    .with_flag_write(FlagWrite::All),
            };
            emulated!(inst, il, op);
        }
        Instruction::Dint(_) => {
            // If GIE flag is ever exposed this should clear it
        }
        Instruction::Eint(_) => {
            // If GIE flag is ever exposed this should set it
        }
        Instruction::Inc(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => il.sx(
                    2,
                    il.add(size, dest, il.const_int(size, 1))
                        .with_flag_write(FlagWrite::All),
                ),
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => il
                    .add(size, dest, il.const_int(size, 1))
                    .with_flag_write(FlagWrite::All),
            };
            emulated!(inst, il, op);
        }
        Instruction::Incd(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => il.sx(
                    2,
                    il.add(size, dest, il.const_int(size, 2))
                        .with_flag_write(FlagWrite::All),
                ),
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => il
                    .add(size, dest, il.const_int(size, 2))
                    .with_flag_write(FlagWrite::All),
            };
            emulated!(inst, il, op);
        }
        Instruction::Inv(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => {
                    il.sx(2, il.not(size, dest).with_flag_write(FlagWrite::Nvz))
                }
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                    il.not(size, dest).with_flag_write(FlagWrite::Nvz)
                }
            };
            emulated!(inst, il, op);
            il.set_flag(Flag::C, il.not(0, il.flag(Flag::Z))).append();
        }
        Instruction::Nop(_) => {
            il.nop().append();
        }
        Instruction::Pop(inst) => {
            if let Some(Operand::RegisterDirect(r)) = inst.destination() {
                let size = match inst.operand_width() {
                    Some(width) => width_to_size(width),
                    None => 2,
                };
                il.set_reg(size, Register::try_from(*r as u32).unwrap(), il.pop(2))
                    .append();
            } else {
                info!("pop: invalid destination operand");
            }
        }
        Instruction::Ret(_) => {
            il.ret(il.pop(2)).append();
        }
        Instruction::Rla(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let src = il.const_int(size, 1);
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => {
                    il.sx(2, il.rol(size, dest, src).with_flag_write(FlagWrite::All))
                }
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                    il.rol(size, dest, src).with_flag_write(FlagWrite::All)
                }
            };
            emulated!(inst, il, op);
        }
        Instruction::Rlc(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let src = il.const_int(size, 1);
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => {
                    il.sx(2, il.rlc(size, dest, src).with_flag_write(FlagWrite::All))
                }
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                    il.rlc(size, dest, src).with_flag_write(FlagWrite::All)
                }
            };
            emulated!(inst, il, op);
        }
        Instruction::Sbc(inst) => {
            // SBC dst is emulated as SUBC #0, dst
            // Effectively: dst = dst - carry
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            let carry = il.flag(Flag::C);
            // SBC subtracts zero with borrow, which is just subtracting the borrow
            let op = match inst.operand_width() {
                Some(OperandWidth::Byte) => {
                    il.sx(2, il.sbb(size, dest, il.const_int(size, 0), carry)
                        .with_flag_write(FlagWrite::All))
                }
                Some(OperandWidth::Word) | Some(OperandWidth::Address) | None => {
                    il.sbb(size, dest, il.const_int(size, 0), carry)
                        .with_flag_write(FlagWrite::All)
                }
            };
            emulated!(inst, il, op);
        }
        Instruction::Setc(_) => {
            // TODO: should we lift setting the C bit in the SR register as well?
            il.set_flag(Flag::C, il.const_int(0, 1)).append();
        }
        Instruction::Setn(_) => {
            // TODO: should we lift setting the N bit in the SR register as well?
            il.set_flag(Flag::N, il.const_int(0, 1)).append();
        }
        Instruction::Setz(_) => {
            // TODO: should we lift setting the Z bit in the SR register as well?
            il.set_flag(Flag::Z, il.const_int(0, 1)).append();
        }
        Instruction::Tst(inst) => {
            let size = match inst.operand_width() {
                Some(width) => width_to_size(width),
                None => 2,
            };
            let dest = lift_source_operand(&inst.destination().unwrap(), size, il);
            il.sub(size, dest, il.const_int(size, 0))
                .with_flag_write(FlagWrite::Nz)
                .append();
            il.set_flag(Flag::V, il.const_int(0, 0)).append();
            il.set_flag(Flag::C, il.const_int(0, 1)).append();
        }
        // MSP430X extended instructions
        Instruction::Mova(inst) => {
            // 20-bit move address - always uses 3-byte size
            let src = lift_source_operand(inst.source(), 3, il);
            msp430x_address_write(inst.destination(), il, src);
        }
        Instruction::Cmpa(inst) => {
            // 20-bit compare address
            let src = lift_source_operand(inst.source(), 3, il);
            let dest = lift_source_operand(inst.destination(), 3, il);
            il.sub(3, dest, src)
                .with_flag_write(FlagWrite::All)
                .append();
        }
        Instruction::Adda(inst) => {
            // 20-bit add address
            let src = lift_source_operand(inst.source(), 3, il);
            let dest = lift_source_operand(inst.destination(), 3, il);
            let op = il.add(3, src, dest).with_flag_write(FlagWrite::All).build();
            msp430x_address_write(inst.destination(), il, op);
        }
        Instruction::Suba(inst) => {
            // 20-bit subtract address
            let src = lift_source_operand(inst.source(), 3, il);
            let dest = lift_source_operand(inst.destination(), 3, il);
            let op = il.sub(3, dest, src).with_flag_write(FlagWrite::All).build();
            msp430x_address_write(inst.destination(), il, op);
        }
        Instruction::Bra(inst) => {
            // 20-bit unconditional branch (jump)
            let dest = if let Operand::Immediate20(dest) = inst.destination() {
                il.const_ptr(*dest as u64)
            } else if let Operand::Immediate(dest) = inst.destination() {
                il.const_ptr(*dest as u64)
            } else if let Operand::Absolute20(dest) = inst.destination() {
                il.const_ptr(*dest as u64)
            } else if let Operand::Absolute(dest) = inst.destination() {
                il.const_ptr(*dest as u64)
            } else {
                lift_source_operand(inst.destination(), 3, il)
            };
            il.jump(dest).append();
        }
        Instruction::Calla(inst) => {
            // 20-bit call - similar to CALL but with 3-byte addressing
            let src = if let Operand::Immediate20(src) = inst.destination() {
                il.const_ptr(*src as u64)
            } else if let Operand::Immediate(src) = inst.destination() {
                il.const_ptr(*src as u64)
            } else {
                lift_source_operand(inst.destination(), 3, il)
            };
            il.call(src).append();
        }
        Instruction::Reta(_inst) => {
            // 20-bit return - pop 3 bytes from stack
            il.ret(il.pop(3)).append();
        }
        Instruction::Rrcm(inst) => {
            // Rotate right through carry multiple times
            let count = inst.count();
            let reg = Register::try_from(inst.register() as u32).unwrap();
            let size = if inst.is_address() { 3 } else { 2 };
            let reg_val = il.reg(size, reg);
            let shift_amount = il.const_int(size, count as u64);
            let op = il.rrc(size, reg_val, shift_amount).with_flag_write(FlagWrite::All);
            il.set_reg(size, reg, op).append();
        }
        Instruction::Rram(inst) => {
            // Rotate right arithmetic multiple times (arithmetic shift right)
            let count = inst.count();
            let reg = Register::try_from(inst.register() as u32).unwrap();
            let size = if inst.is_address() { 3 } else { 2 };
            let reg_val = il.reg(size, reg);
            let shift_amount = il.const_int(size, count as u64);
            let op = il.asr(size, reg_val, shift_amount).with_flag_write(FlagWrite::Cnz);
            il.set_reg(size, reg, op).append();
            il.set_flag(Flag::V, il.const_int(0, 0)).append();
        }
        Instruction::Rlam(inst) => {
            // Rotate left arithmetic multiple times (logical shift left)
            let count = inst.count();
            let reg = Register::try_from(inst.register() as u32).unwrap();
            let size = if inst.is_address() { 3 } else { 2 };
            let reg_val = il.reg(size, reg);
            let shift_amount = il.const_int(size, count as u64);
            let op = il.lsl(size, reg_val, shift_amount).with_flag_write(FlagWrite::All);
            il.set_reg(size, reg, op).append();
        }
        Instruction::Rrum(inst) => {
            // Rotate right unsigned multiple times (logical shift right)
            let count = inst.count();
            let reg = Register::try_from(inst.register() as u32).unwrap();
            let size = if inst.is_address() { 3 } else { 2 };
            let reg_val = il.reg(size, reg);
            let shift_amount = il.const_int(size, count as u64);
            let op = il.lsr(size, reg_val, shift_amount).with_flag_write(FlagWrite::Cnz);
            il.set_reg(size, reg, op).append();
            il.set_flag(Flag::V, il.const_int(0, 0)).append();
        }
        Instruction::Pushm(inst) => {
            // Push multiple registers to stack
            let count = inst.count();
            let end_reg = inst.register();
            let size = if inst.is_address() { 3 } else { 2 };

            // Push registers in descending order from end_reg to (end_reg - count + 1)
            for i in 0..count {
                let reg_num = end_reg.wrapping_sub(i);
                if let Ok(reg) = Register::try_from(reg_num as u32) {
                    il.push(size, il.reg(size, reg)).append();
                }
            }
        }
        Instruction::Popm(inst) => {
            // Pop multiple registers from stack
            let count = inst.count();
            let end_reg = inst.register();
            let size = if inst.is_address() { 3 } else { 2 };

            // Pop registers in ascending order to (end_reg - count + 1) to end_reg
            for i in (0..count).rev() {
                let reg_num = end_reg.wrapping_sub(i);
                if let Ok(reg) = Register::try_from(reg_num as u32) {
                    il.set_reg(size, reg, il.pop(size)).append();
                }
            }
        }
    }
}

fn lift_source_operand<'a>(
    operand: &Operand,
    size: usize,
    il: &'a LowLevelILMutableFunction,
) -> LowLevelILMutableExpression<'a, ValueExpr> {
    match operand {
        Operand::RegisterDirect(r) => il.reg(size, Register::try_from(*r as u32).unwrap()),
        Operand::Indexed((r, offset)) => il
            .load(
                size,
                il.add(
                    2,
                    il.reg(2, Register::try_from(*r as u32).unwrap()),
                    il.const_int(2, *offset as u64),
                ),
            )
            .build(),
        // MSP430X 20-bit indexed addressing
        Operand::Indexed20((r, offset)) => il
            .load(
                size,
                il.add(
                    3,
                    il.reg(3, Register::try_from(*r as u32).unwrap()),
                    il.const_int(3, *offset as u64),
                ),
            )
            .build(),
        // should we add offset to addr here rather than lifting to the register since we know where PC is?
        Operand::Symbolic(offset) => il
            .load(
                size,
                il.add(2, il.reg(2, Register::Pc), il.const_int(2, *offset as u64)),
            )
            .build(),
        // MSP430X 20-bit symbolic addressing
        Operand::Symbolic20(offset) => il
            .load(
                size,
                il.add(3, il.reg(3, Register::Pc), il.const_int(3, *offset as u64)),
            )
            .build(),
        Operand::Absolute(addr) => il.load(size, il.const_ptr(*addr as u64)).build(),
        // MSP430X 20-bit absolute addressing
        Operand::Absolute20(addr) => il.load(size, il.const_ptr(*addr as u64)).build(),
        // these are the same, we need to autoincrement in a separate il instruction
        Operand::RegisterIndirect(r) | Operand::RegisterIndirectAutoIncrement(r) => il
            .load(size, il.reg(2, Register::try_from(*r as u32).unwrap()))
            .build(),
        Operand::Immediate(val) => il.const_int(size, *val as u64),
        // MSP430X 20-bit immediate
        Operand::Immediate20(val) => il.const_int(size, *val as u64),
        Operand::Constant(val) => il.const_int(size, *val as u64),
    }
}

fn width_to_size(width: &OperandWidth) -> usize {
    match width {
        OperandWidth::Byte => 1,
        OperandWidth::Word => 2,
        OperandWidth::Address => 3, // 20-bit for MSP430X
    }
}

// Helper function to write to MSP430X 20-bit address operands
fn msp430x_address_write<'a>(
    operand: &Operand,
    il: &'a LowLevelILMutableFunction,
    value: LowLevelILMutableExpression<'a, ValueExpr>,
) {
    match operand {
        Operand::RegisterDirect(r) => {
            il.set_reg(3, Register::try_from(*r as u32).unwrap(), value)
                .append();
        }
        Operand::Indexed20((r, offset)) => {
            il.store(
                3,
                il.add(
                    3,
                    il.reg(3, Register::try_from(*r as u32).unwrap()),
                    il.const_int(3, *offset as u64),
                ),
                value,
            )
            .append();
        }
        Operand::Symbolic20(offset) => {
            il.store(3, il.add(3, il.reg(3, Register::Pc), *offset as u64), value)
                .append();
        }
        Operand::Absolute20(addr) => {
            il.store(3, il.const_ptr(*addr as u64), value).append();
        }
        // 16-bit variants also supported for compatibility
        Operand::Indexed((r, offset)) => {
            il.store(
                3,
                il.add(
                    3,
                    il.reg(3, Register::try_from(*r as u32).unwrap()),
                    il.const_int(3, *offset as u64),
                ),
                value,
            )
            .append();
        }
        Operand::Symbolic(offset) => {
            il.store(3, il.add(3, il.reg(3, Register::Pc), *offset as u64), value)
                .append();
        }
        Operand::Absolute(addr) => {
            il.store(3, il.const_ptr(*addr as u64), value).append();
        }
        _ => {
            // For other operand types, this shouldn't happen in address instructions
            unreachable!("Unexpected operand type for MSP430X address write");
        }
    }
}
