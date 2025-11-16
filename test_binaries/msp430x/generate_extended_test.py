#!/usr/bin/env python3
"""
Generate MSP430X test binary with extension word instructions

This script creates a minimal ELF binary containing MSP430X instructions
that use extension words (0x18xx prefix) for 20-bit addressing.

Instructions tested:
- MOVA: Move to/from 20-bit address
- CALLA: Call to 20-bit address
- CMPA: Compare with 20-bit operand
- ADDA: Add with 20-bit operand
- Various addressing modes with extension words
"""

import struct
import os

# MSP430X Extension Word Format (0x18xx)
# Bits [15:11] = 00011 (0x18 prefix when shifted)
# Bit [10:9] = Source/destination addressing mode extension
# Bit [8] = Repetition count enable
# Bit [7] = A/L bit (Address/Length)
# Bits [6:0] = Extension data or repetition count

def make_extension_word(src_mode=0, dst_mode=0, al_bit=0, data=0):
    """Create MSP430X extension word (0x18xx format)"""
    word = 0x1800  # Base pattern
    word |= (src_mode & 0x3) << 9  # Bits [10:9]
    word |= (dst_mode & 0x3) << 7   # Actually part of data field
    word |= (al_bit & 0x1) << 6
    word |= (data & 0x3F)
    return word

# MSP430X Instruction Opcodes
# Format 2 (20-bit extensions)
MOVA_INDIRECT_REG = 0x0000  # MOVA @Rs, Rd   (opcode 00 00)
MOVA_INDEXED = 0x0010       # MOVA x(Rs), Rd  (opcode 00 10)
MOVA_IMMEDIATE = 0x0020     # MOVA #imm20, Rd (opcode 00 20)
MOVA_ABSOLUTE = 0x0030      # MOVA &abs20, Rd (opcode 00 30)
MOVA_REG_INDEXED = 0x0040   # MOVA Rs, x(Rd)  (opcode 00 40)
MOVA_REG_ABSOLUTE = 0x0050  # MOVA Rs, &abs20 (opcode 00 50)
MOVA_REG_REG = 0x0060       # MOVA Rs, Rd     (opcode 00 60)
MOVA_IMMEDIATE_REG = 0x0080 # MOVA @Rs+, Rd   (opcode 00 80)

CMPA_IMMEDIATE = 0x0090     # CMPA #imm20, Rd (opcode 00 90)
ADDA_IMMEDIATE = 0x00A0     # ADDA #imm20, Rd (opcode 00 A0)
SUBA_IMMEDIATE = 0x00B0     # SUBA #imm20, Rd (opcode 00 B0)

CALLA_REG = 0x1340          # CALLA Rs        (opcode 13 40)
CALLA_INDEXED = 0x1350      # CALLA x(Rs)     (opcode 13 50)
CALLA_INDIRECT = 0x1360     # CALLA @Rs       (opcode 13 60)
CALLA_INDIRECT_INC = 0x1370 # CALLA @Rs+      (opcode 13 70)
CALLA_ABSOLUTE = 0x1380     # CALLA &abs20    (opcode 13 80)
CALLA_IMMEDIATE = 0x13B0    # CALLA #imm20    (opcode 13 B0)
CALLA_SYMBOLIC = 0x1390     # CALLA dst       (opcode 13 90)

def encode_mova_immediate_reg(imm20, rd):
    """Encode: MOVA #imm20, Rd"""
    # Extension word contains upper 4 bits
    ext = make_extension_word(data=(imm20 >> 16) & 0xF)
    # Instruction word
    instr = MOVA_IMMEDIATE | (rd & 0xF)
    # Immediate word (lower 16 bits)
    imm_word = imm20 & 0xFFFF
    return [ext, instr, imm_word]

def encode_mova_absolute_reg(abs20, rd):
    """Encode: MOVA &abs20, Rd"""
    ext = make_extension_word(data=(abs20 >> 16) & 0xF)
    instr = MOVA_ABSOLUTE | (rd & 0xF)
    abs_word = abs20 & 0xFFFF
    return [ext, instr, abs_word]

def encode_mova_reg_absolute(rs, abs20):
    """Encode: MOVA Rs, &abs20"""
    ext = make_extension_word(data=(abs20 >> 16) & 0xF)
    instr = MOVA_REG_ABSOLUTE | ((rs & 0xF) << 4)
    abs_word = abs20 & 0xFFFF
    return [ext, instr, abs_word]

def encode_mova_indexed_reg(offset, rs, rd):
    """Encode: MOVA x(Rs), Rd"""
    ext = make_extension_word(data=(offset >> 16) & 0xF)
    instr = MOVA_INDEXED | ((rs & 0xF) << 4) | (rd & 0xF)
    offset_word = offset & 0xFFFF
    return [ext, instr, offset_word]

def encode_cmpa_immediate_reg(imm20, rd):
    """Encode: CMPA #imm20, Rd"""
    ext = make_extension_word(data=(imm20 >> 16) & 0xF)
    instr = CMPA_IMMEDIATE | (rd & 0xF)
    imm_word = imm20 & 0xFFFF
    return [ext, instr, imm_word]

def encode_adda_immediate_reg(imm20, rd):
    """Encode: ADDA #imm20, Rd"""
    ext = make_extension_word(data=(imm20 >> 16) & 0xF)
    instr = ADDA_IMMEDIATE | (rd & 0xF)
    imm_word = imm20 & 0xFFFF
    return [ext, instr, imm_word]

def encode_calla_immediate(imm20):
    """Encode: CALLA #imm20"""
    ext = make_extension_word(data=(imm20 >> 16) & 0xF)
    instr = CALLA_IMMEDIATE
    imm_word = imm20 & 0xFFFF
    return [ext, instr, imm_word]

def encode_calla_absolute(abs20):
    """Encode: CALLA &abs20"""
    ext = make_extension_word(data=(abs20 >> 16) & 0xF)
    instr = CALLA_ABSOLUTE
    abs_word = abs20 & 0xFFFF
    return [ext, instr, abs_word]

def encode_calla_reg(rs):
    """Encode: CALLA Rs"""
    instr = CALLA_REG | (rs & 0xF)
    return [instr]

def encode_reta():
    """Encode: RETA (return from subroutine with 20-bit address)"""
    return [0x0110]

# Build the instruction sequence
instructions = []

# Address 0x8000: Entry point
# Test various MOVA instructions
addr = 0x8000

# MOVA #0x12345, r15  - Move 20-bit immediate to r15
instructions.extend(encode_mova_immediate_reg(0x12345, 15))

# MOVA &0x10000, r14  - Move from 20-bit absolute address
instructions.extend(encode_mova_absolute_reg(0x10000, 14))

# MOVA r15, &0x10004  - Move to 20-bit absolute address
instructions.extend(encode_mova_reg_absolute(15, 0x10004))

# MOVA 0x1000(r10), r13  - Move with 20-bit indexed addressing
instructions.extend(encode_mova_indexed_reg(0x1000, 10, 13))

# CMPA #0x12345, r15  - Compare 20-bit immediate
instructions.extend(encode_cmpa_immediate_reg(0x12345, 15))

# ADDA #0x1000, r15  - Add 20-bit immediate
instructions.extend(encode_adda_immediate_reg(0x1000, 15))

# CALLA #0x8050  - Call to 20-bit immediate address
instructions.extend(encode_calla_immediate(0x8050))

# Address for subroutine (calculate offset from above)
# Pad to reach 0x8050
current_addr = 0x8000 + len(instructions) * 2
while current_addr < 0x8050:
    instructions.append(0x4303)  # NOP (MOV #0, r3 - constant generator)
    current_addr += 2

# Subroutine at 0x8050
# CALLA r12  - Call to address in r12
instructions.extend(encode_calla_reg(12))

# CALLA &0x8060  - Call to absolute 20-bit address
instructions.extend(encode_calla_absolute(0x8060))

# Pad to 0x8060
current_addr = 0x8000 + len(instructions) * 2
while current_addr < 0x8060:
    instructions.append(0x4303)  # NOP
    current_addr += 2

# Function that returns with RETA
instructions.extend(encode_reta())

# Add a few more NOPs and final infinite loop
instructions.append(0x4303)  # NOP
instructions.append(0x3FFF)  # JMP $-2 (infinite loop)

# Convert to bytes
code = b''.join(struct.pack('<H', instr) for instr in instructions)

# Create minimal ELF header for MSP430
# ELF header (52 bytes)
e_ident = b'\x7fELF'  # Magic
e_ident += b'\x01'    # 32-bit
e_ident += b'\x01'    # Little endian
e_ident += b'\x01'    # ELF version
e_ident += b'\x00' * 9  # Padding

elf_header = e_ident
elf_header += struct.pack('<H', 2)      # e_type = ET_EXEC
elf_header += struct.pack('<H', 0x69)   # e_machine = EM_MSP430 (105)
elf_header += struct.pack('<I', 1)      # e_version
elf_header += struct.pack('<I', 0x8000) # e_entry
elf_header += struct.pack('<I', 52)     # e_phoff (program header offset)
elf_header += struct.pack('<I', 0)      # e_shoff (no section headers)
elf_header += struct.pack('<I', 0)      # e_flags
elf_header += struct.pack('<H', 52)     # e_ehsize
elf_header += struct.pack('<H', 32)     # e_phentsize
elf_header += struct.pack('<H', 1)      # e_phnum
elf_header += struct.pack('<H', 0)      # e_shentsize
elf_header += struct.pack('<H', 0)      # e_shnum
elf_header += struct.pack('<H', 0)      # e_shstrndx

# Program header (32 bytes for 32-bit ELF)
program_header = struct.pack('<I', 1)   # p_type = PT_LOAD
program_header += struct.pack('<I', 52 + 32)  # p_offset (after headers)
program_header += struct.pack('<I', 0x8000)   # p_vaddr
program_header += struct.pack('<I', 0x8000)   # p_paddr
program_header += struct.pack('<I', len(code)) # p_filesz
program_header += struct.pack('<I', len(code)) # p_memsz
program_header += struct.pack('<I', 5)  # p_flags = R+X
program_header += struct.pack('<I', 1)  # p_align

# Write the ELF file
output_path = '/home/user/binaryninja-api/test_binaries/msp430x/test_extended.elf'
with open(output_path, 'wb') as f:
    f.write(elf_header)
    f.write(program_header)
    f.write(code)

print(f"Generated: {output_path}")
print(f"Code size: {len(code)} bytes ({len(instructions)} instructions)")
print(f"Entry point: 0x8000")
print()
print("Instruction summary:")
print(f"  - MOVA #0x12345, r15      @ 0x8000")
print(f"  - MOVA &0x10000, r14      @ 0x8006")
print(f"  - MOVA r15, &0x10004      @ 0x800C")
print(f"  - MOVA 0x1000(r10), r13   @ 0x8012")
print(f"  - CMPA #0x12345, r15      @ 0x8018")
print(f"  - ADDA #0x1000, r15       @ 0x801E")
print(f"  - CALLA #0x8050           @ 0x8024")
print(f"  - CALLA r12               @ 0x8050")
print(f"  - CALLA &0x8060           @ 0x8052")
print(f"  - RETA                    @ 0x8060")

# Verify extension words
print()
print("Verification - checking for extension words (0x18xx):")
for i, instr in enumerate(instructions[:20]):  # Check first 20 instructions
    addr = 0x8000 + i * 2
    if (instr & 0xFC00) == 0x1800:
        print(f"  0x{addr:04X}: 0x{instr:04X} (extension word)")
