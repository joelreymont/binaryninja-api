#!/usr/bin/env python3
"""
RISC-V JALR Branch Detection Tests

Tests for fix to issue #6273 - RISC-V JALR branch detection for indirect calls.

This test validates that JALR instructions are properly classified:
- jalr x0, rs1, imm  -> Unresolved branch
- jalr x1, x1, 0     -> Function return
- jalr rd, rs1, imm  -> Indirect call (when rd != 0)

Run with: python3 test_riscv_jalr.py
Requires: Binary Ninja installation with RISC-V architecture
"""

import sys
import struct

try:
    import binaryninja
    from binaryninja import binaryview, Architecture
except ImportError:
    print("ERROR: Binary Ninja not found. This test requires Binary Ninja to be installed.", file=sys.stderr)
    sys.exit(1)

def encode_jalr(rd, rs1, imm):
    """
    Encode a RISC-V JALR instruction.
    Format: imm[11:0] | rs1[4:0] | 000 | rd[4:0] | 1100111
    """
    opcode = 0b1100111
    funct3 = 0b000
    imm12 = imm & 0xFFF
    instr = (imm12 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
    return struct.pack('<I', instr)

# Test cases: (instruction_bytes, description, expected_branch_type)
test_cases = [
    # Issue #6273: jalr rd, rs, imm where rd != 0 should be indirect call
    (encode_jalr(5, 10, 0), "jalr x5, x10, 0 - indirect call", "IndirectBranch"),
    (encode_jalr(10, 5, 4), "jalr x10, x5, 4 - indirect call", "IndirectBranch"),
    (encode_jalr(3, 7, 8), "jalr x3, x7, 8 - indirect call", "IndirectBranch"),

    # Standard return: jalr x0, x1, 0 (ret pseudo-instruction)
    (encode_jalr(0, 1, 0), "jalr x0, x1, 0 - function return (ret)", "FunctionReturn"),

    # Unresolved branch: jalr x0, rs, imm where rs != x1
    (encode_jalr(0, 5, 0), "jalr x0, x5, 0 - unresolved branch", "UnresolvedBranch"),
    (encode_jalr(0, 10, 8), "jalr x0, x10, 8 - unresolved branch", "UnresolvedBranch"),

    # Edge cases
    (encode_jalr(1, 1, 0), "jalr x1, x1, 0 - return (alternative)", "FunctionReturn"),
    (encode_jalr(2, 2, 0), "jalr x2, x2, 0 - indirect call (self-update)", "IndirectBranch"),
]

def test_jalr_branch_detection():
    """Test JALR branch detection"""
    arch = Architecture['riscv']
    if not arch:
        print("ERROR: RISC-V architecture not found", file=sys.stderr)
        return False

    print(f"Testing RISC-V JALR branch detection ({len(test_cases)} tests)...", file=sys.stderr)

    passed = 0
    failed = 0

    for test_i, (instr_bytes, description, expected_type) in enumerate(test_cases):
        # Get instruction info
        info = arch.get_instruction_info(instr_bytes, 0)

        if not info:
            print(f"FAIL Test {test_i + 1}: {description}")
            print(f"  Could not get instruction info")
            failed += 1
            continue

        # Check if instruction has branches
        has_branch = len(info.branches) > 0

        if expected_type == "IndirectBranch" or expected_type == "UnresolvedBranch" or expected_type == "FunctionReturn":
            if not has_branch:
                print(f"FAIL Test {test_i + 1}: {description}")
                print(f"  Expected branch but none found")
                print(f"  Instruction bytes: {instr_bytes.hex()}")
                failed += 1
                continue

            # Check branch type
            branch = info.branches[0]
            actual_type = str(branch.type).split('.')[-1]  # Get enum name

            if actual_type != expected_type:
                print(f"FAIL Test {test_i + 1}: {description}")
                print(f"  Expected: {expected_type}")
                print(f"  Actual: {actual_type}")
                print(f"  Instruction bytes: {instr_bytes.hex()}")
                failed += 1
                continue

        passed += 1
        print(f"PASS Test {test_i + 1}: {description}")

    print(f"\nResults: {passed} passed, {failed} failed out of {len(test_cases)} tests")
    return failed == 0

def test_jalr_il_lifting():
    """Test JALR IL lifting"""
    arch = Architecture['riscv']
    if not arch:
        print("ERROR: RISC-V architecture not found", file=sys.stderr)
        return False

    print("\nTesting RISC-V JALR IL lifting...", file=sys.stderr)

    # Test cases: (instruction_bytes, description, expected_il_contains)
    il_tests = [
        (encode_jalr(5, 10, 0), "jalr x5, x10, 0", "LLIL_CALL"),
        (encode_jalr(0, 1, 0), "jalr x0, x1, 0 (ret)", "LLIL_RET"),
        (encode_jalr(0, 5, 0), "jalr x0, x5, 0", "LLIL_JUMP"),
    ]

    passed = 0
    failed = 0

    for test_i, (instr_bytes, description, expected_il) in enumerate(il_tests):
        # Create a minimal binary view
        PROLOG = b''
        EPILOG = b'\x13\x05\xd0\xde'  # li a0, 0xdead (marker instruction)

        data = PROLOG + instr_bytes + EPILOG
        bv = binaryview.BinaryView.new(data)
        bv.platform = arch.standalone_platform
        bv.add_function(0)

        if len(bv.functions) == 0:
            print(f"SKIP Test {test_i + 1}: {description} - no function created")
            continue

        # Get IL
        il_str_parts = []
        for block in bv.functions[0].lifted_il:
            for il in block:
                il_str_parts.append(str(il))

        il_str = '; '.join(il_str_parts)

        if expected_il in il_str:
            passed += 1
            print(f"PASS IL Test {test_i + 1}: {description}")
            print(f"  IL contains: {expected_il}")
        else:
            failed += 1
            print(f"FAIL IL Test {test_i + 1}: {description}")
            print(f"  Expected IL to contain: {expected_il}")
            print(f"  Actual IL: {il_str}")

    print(f"\nIL Results: {passed} passed, {failed} failed out of {len(il_tests)} tests")
    return failed == 0

if __name__ == '__main__':
    success = True

    # Run branch detection tests
    if not test_jalr_branch_detection():
        success = False

    # Run IL lifting tests
    if not test_jalr_il_lifting():
        success = False

    sys.exit(0 if success else 1)
