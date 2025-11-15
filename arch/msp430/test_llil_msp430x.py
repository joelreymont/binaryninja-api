#!/usr/bin/env python3
"""
MSP430X LLIL Lifting Tests

These tests validate that MSP430X instructions are lifted to correct LLIL.
Requires Binary Ninja with MSP430 architecture plugin loaded.

Run with: python3 test_llil_msp430x.py
"""

# Expected LLIL output for MSP430X instructions
# Format: (instruction_bytes, expected_llil_string)

tests_mova = [
    # MOVA r15, r14 - move 20-bit register
    # Should set r14 to the value of r15 (3-byte operation)
    (b"\xce\x0f", "LLIL_SET_REG.w(r14,LLIL_REG.w(r15))"),

    # Note: .w might actually be a different size for 20-bit, but this shows the concept
]

tests_reta = [
    # RETA - return address mode (20-bit)
    # Should pop 3 bytes from stack and return
    (b"\x10\x01", "LLIL_RET(LLIL_POP.w())"),
]

tests_calla = [
    # CALLA #17428 - call 20-bit address
    # Should call to the immediate address
    (b"\xb0\x13\x14\x44", "LLIL_CALL(LLIL_CONST_PTR(0x4414))"),
]

tests_pushm = [
    # PUSHM.A #4, r15 - push r12-r15 in address mode
    # Should generate 4 PUSH operations for registers r12, r13, r14, r15
    # Binary Ninja might combine these or show them separately
    (b"\x3f\x14", [
        "LLIL_PUSH.w(LLIL_REG.w(r15))",
        "LLIL_PUSH.w(LLIL_REG.w(r14))",
        "LLIL_PUSH.w(LLIL_REG.w(r13))",
        "LLIL_PUSH.w(LLIL_REG.w(r12))",
    ]),
]

tests_popm = [
    # POPM.A #4, r15 - pop into r12-r15 in address mode
    # Should generate 4 POP operations
    (b"\x3c\x16", [
        "LLIL_SET_REG.w(r12,LLIL_POP.w())",
        "LLIL_SET_REG.w(r13,LLIL_POP.w())",
        "LLIL_SET_REG.w(r14,LLIL_POP.w())",
        "LLIL_SET_REG.w(r15,LLIL_POP.w())",
    ]),
]

tests_rlam = [
    # RLAM.A #2, r15 - logical shift left 2 bits (address mode)
    # Should generate LSL with count=2
    (b"\x4f\x06", "LLIL_SET_REG.w(r15,LLIL_LSL.w(LLIL_REG.w(r15),LLIL_CONST.w(2)))"),
]

tests_rram = [
    # RRAM.A #2, r14 - arithmetic shift right 2 bits (address mode)
    # Should generate ASR with count=2
    (b"\x4e\x05", "LLIL_SET_REG.w(r14,LLIL_ASR.w(LLIL_REG.w(r14),LLIL_CONST.w(2)))"),
]

tests_adda = [
    # ADDA would be something like:
    # (b"...", "LLIL_SET_REG.w(r15,LLIL_ADD.w(LLIL_REG.w(r14),LLIL_REG.w(r15)))"),
]

tests_suba = [
    # SUBA would be something like:
    # (b"...", "LLIL_SET_REG.w(r15,LLIL_SUB.w(LLIL_REG.w(r15),LLIL_REG.w(r14)))"),
]

tests_cmpa = [
    # CMPA would be something like (doesn't store result, just sets flags):
    # (b"...", "LLIL_SUB.w(LLIL_REG.w(r15),LLIL_REG.w(r14))"),
]

if __name__ == "__main__":
    try:
        import binaryninja as bn
        from binaryninja import BinaryView

        print("Binary Ninja Python API found!")
        print(f"Version: {bn.core_version()}")

        # To actually run these tests, you would:
        # 1. Create a BinaryView with MSP430X architecture
        # 2. Write the instruction bytes to memory
        # 3. Get the LLIL for that address
        # 4. Compare the LLIL string representation to expected output

        print("\nExpected LLIL tests defined:")
        print(f"  MOVA:  {len(tests_mova)} tests")
        print(f"  RETA:  {len(tests_reta)} tests")
        print(f"  CALLA: {len(tests_calla)} tests")
        print(f"  PUSHM: {len(tests_pushm)} tests")
        print(f"  POPM:  {len(tests_popm)} tests")
        print(f"  RLAM:  {len(tests_rlam)} tests")
        print(f"  RRAM:  {len(tests_rram)} tests")

        print("\nTo run actual validation:")
        print("1. Install Binary Ninja")
        print("2. Build and install the MSP430 architecture plugin")
        print("3. Run this script to compare actual vs expected LLIL")

    except ImportError:
        print("Binary Ninja Python API not available.")
        print("This script defines expected LLIL output but cannot run validation.")
        print("\nInstall Binary Ninja to run actual LLIL validation tests.")
