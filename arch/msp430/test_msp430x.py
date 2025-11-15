#!/usr/bin/env python
"""
MSP430X Architecture Test Suite

Tests MSP430X extended instruction decoding and LLIL lifting.
Each test is (bytes, expected_llil_string) where bytes are the raw
instruction encoding.
"""

# MSP430X Address Instructions (20-bit operations)
tests_mova = [
    # MOVA @r5, r15 - move 20-bit address from memory pointed by r5 to r15
    # Opcode: 0x0005 0x0F00
    (b"\x00\x05\x00\x0F", "LLIL_SET_REG.w(r15,LLIL_LOAD.w(LLIL_REG.w(r5)))"),

    # MOVA #0x12345, r10 - move immediate 20-bit value to r10
    # This tests immediate20 operand handling
    # Format depends on encoding - placeholder for now
]

tests_cmpa = [
    # CMPA #0x10000, r15 - compare 20-bit immediate with r15
    # Should generate SUB without storing result, only updating flags
    (b"\x90\x13\x00\x00\x01", "LLIL_SUB.w(LLIL_REG.w(r15),LLIL_CONST.w(0x10000))"),
]

tests_adda = [
    # ADDA r14, r15 - add 20-bit register values
    # Should generate ADD with flag updates
]

tests_suba = [
    # SUBA r14, r15 - subtract 20-bit register values
    # Should generate SUB with flag updates
]

# MSP430X Control Flow
tests_calla = [
    # CALLA #0x10000 - call to 20-bit immediate address
    # Should generate CALL with 20-bit target
]

tests_reta = [
    # RETA - return from subroutine (20-bit)
    # Should generate RET with 3-byte pop
    (b"\x00\x13", "LLIL_RET(LLIL_POP.w())"),
]

# MSP430X Rotate/Shift Multiple
tests_rrcm = [
    # RRCM.A #2, r15 - rotate right through carry 2 bits, 20-bit mode
    # Should generate RRC with count=2
]

tests_rram = [
    # RRAM.W #3, r10 - arithmetic shift right 3 bits, 16-bit mode
    # Should generate ASR (arithmetic shift right)
]

tests_rlam = [
    # RLAM.A #4, r15 - logical shift left 4 bits, 20-bit mode
    # Should generate LSL (logical shift left)
]

tests_rrum = [
    # RRUM.W #1, r14 - logical shift right 1 bit, 16-bit mode
    # Should generate LSR (logical shift right)
]

# MSP430X Stack Operations
tests_pushm = [
    # PUSHM.A #3, r15 - push r13, r14, r15 (3 registers in 20-bit mode)
    # Should generate 3 PUSH operations
]

tests_popm = [
    # POPM.W #2, r10 - pop r9, r10 (2 registers in 16-bit mode)
    # Should generate 2 POP operations
]

# Base MSP430 regression tests (ensure we didn't break existing functionality)
tests_base_msp430 = [
    # MOV.W r15, r14
    (b"\x0F\x4E", "LLIL_SET_REG.w(r14,LLIL_REG.w(r15))"),

    # ADD.W r14, r15
    (b"\x0E\x5F", "LLIL_SET_REG.w(r15,LLIL_ADD.w(LLIL_REG.w(r14),LLIL_REG.w(r15)))"),

    # JMP $+2 (relative jump)
    (b"\x3C\x00", "LLIL_JUMP(LLIL_CONST_PTR(current_addr+2))"),
]

all_tests = (
    tests_mova +
    tests_cmpa +
    tests_adda +
    tests_suba +
    tests_calla +
    tests_reta +
    tests_rrcm +
    tests_rram +
    tests_rlam +
    tests_rrum +
    tests_pushm +
    tests_popm +
    tests_base_msp430
)

if __name__ == "__main__":
    print(f"MSP430X Test Suite: {len(all_tests)} test cases defined")
    print("\nNote: These tests require Binary Ninja Python API to run.")
    print("Run with: python3 -m pytest test_msp430x.py")
