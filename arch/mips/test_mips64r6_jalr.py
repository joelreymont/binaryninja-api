#!/usr/bin/env python3
"""
MIPS64 Release 6 JALR Tests

Tests for fix to issue #7355 - MIPS64R6 JALR return recognition.

MIPS64 Release 6 removed the JR instruction and maps it to JALR:
- jalr $zero, $ra  -> Function return (formerly jr $ra)
- jalr $zero, $rs  -> Indirect jump (formerly jr $rs)
- jalr $rd, $rs    -> Indirect call

This test validates proper detection of these variants.

Run with: python3 test_mips64r6_jalr.py
Requires: Binary Ninja installation with MIPS architecture
"""

import sys
import struct

try:
    import binaryninja
    from binaryninja import binaryview, Architecture
except ImportError:
    print("ERROR: Binary Ninja not found. This test requires Binary Ninja to be installed.", file=sys.stderr)
    sys.exit(1)

def encode_jalr(rs, rt, rd, hint=0):
    """
    Encode a MIPS JALR instruction.
    Format: SPECIAL | rs | 0 | rd | hint | JALR.HB/JALR
    opcode=000000 rs(5) rt(5)=00000 rd(5) hint(5) funct=001001

    For MIPS R6, JR is encoded as JALR with rd=$zero
    """
    opcode = 0b000000
    rt_field = 0b00000  # Always 0 for JALR
    funct = 0b001001  # JALR

    instr = (opcode << 26) | (rs << 21) | (rt_field << 16) | (rd << 11) | (hint << 6) | funct
    return struct.pack('<I', instr)

# MIPS register numbers
REG_ZERO = 0
REG_RA = 31
REG_T0 = 8
REG_T1 = 9
REG_T2 = 10

# Test cases: (instruction_bytes, description, expected_branch_type)
test_cases = [
    # Issue #7355: MIPS64R6 return (jalr $zero, $ra)
    (encode_jalr(REG_RA, 0, REG_ZERO, 0), "jalr $zero, $ra - function return (R6 jr $ra)", "FunctionReturn"),

    # MIPS64R6 indirect jump (jalr $zero, $rs where rs != $ra)
    (encode_jalr(REG_T0, 0, REG_ZERO, 0), "jalr $zero, $t0 - unresolved branch (R6 jr $t0)", "UnresolvedBranch"),
    (encode_jalr(REG_T1, 0, REG_ZERO, 0), "jalr $zero, $t1 - unresolved branch (R6 jr $t1)", "UnresolvedBranch"),

    # Standard JALR indirect calls (rd != $zero)
    (encode_jalr(REG_T0, 0, REG_RA, 0), "jalr $ra, $t0 - indirect call", "IndirectBranch"),
    (encode_jalr(REG_T1, 0, REG_RA, 0), "jalr $ra, $t1 - indirect call", "IndirectBranch"),
    (encode_jalr(REG_T2, 0, REG_RA, 0), "jalr $ra, $t2 - indirect call", "IndirectBranch"),

    # JALR.HB variants (hint bit set, same semantics)
    (encode_jalr(REG_RA, 0, REG_ZERO, 1), "jalr.hb $zero, $ra - function return with hazard barrier", "FunctionReturn"),
    (encode_jalr(REG_T0, 0, REG_ZERO, 1), "jalr.hb $zero, $t0 - unresolved branch with hazard barrier", "UnresolvedBranch"),
]

def test_mips_jalr_branch_detection():
    """Test MIPS JALR branch detection"""
    # Try both mips64 and mips architectures
    arch = Architecture['mips64'] or Architecture['mips']
    if not arch:
        print("ERROR: MIPS architecture not found", file=sys.stderr)
        return False

    print(f"Testing MIPS64R6 JALR branch detection ({len(test_cases)} tests)...", file=sys.stderr)
    print(f"Using architecture: {arch.name}", file=sys.stderr)

    passed = 0
    failed = 0

    for test_i, (instr_bytes, description, expected_type) in enumerate(test_cases):
        # Get instruction info
        info = arch.get_instruction_info(instr_bytes, 0)

        if not info:
            print(f"FAIL Test {test_i + 1}: {description}")
            print(f"  Could not get instruction info")
            print(f"  Instruction bytes: {instr_bytes.hex()}")
            failed += 1
            continue

        # Check if instruction has branches
        has_branch = len(info.branches) > 0

        if expected_type in ["IndirectBranch", "UnresolvedBranch", "FunctionReturn"]:
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

def test_mips_jalr_il_lifting():
    """Test MIPS JALR IL lifting"""
    arch = Architecture['mips64'] or Architecture['mips']
    if not arch:
        print("ERROR: MIPS architecture not found", file=sys.stderr)
        return False

    print("\nTesting MIPS64R6 JALR IL lifting...", file=sys.stderr)

    # Test cases: (instruction_bytes, description, expected_il_contains)
    il_tests = [
        (encode_jalr(REG_RA, 0, REG_ZERO, 0), "jalr $zero, $ra (return)", "LLIL_RET"),
        (encode_jalr(REG_T0, 0, REG_ZERO, 0), "jalr $zero, $t0 (jump)", "LLIL_JUMP"),
        (encode_jalr(REG_T0, 0, REG_RA, 0), "jalr $ra, $t0 (call)", "LLIL_CALL"),
    ]

    passed = 0
    failed = 0

    for test_i, (instr_bytes, description, expected_il) in enumerate(il_tests):
        # Create a minimal binary view
        # MIPS instructions are in delay slots, need NOP after branch
        NOP = struct.pack('<I', 0)  # MIPS NOP
        EPILOG = struct.pack('<I', 0x3402dead)  # li $v0, 0xdead (marker)

        data = instr_bytes + NOP + EPILOG
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

        # For MIPS, the IL might be in a different format, so check more flexibly
        if expected_il in il_str or expected_il.replace('LLIL_', 'LLIL_') in il_str:
            passed += 1
            print(f"PASS IL Test {test_i + 1}: {description}")
            print(f"  IL contains: {expected_il}")
        else:
            failed += 1
            print(f"FAIL IL Test {test_i + 1}: {description}")
            print(f"  Expected IL to contain: {expected_il}")
            print(f"  Actual IL: {il_str[:200]}...")

    print(f"\nIL Results: {passed} passed, {failed} failed out of {len(il_tests)} tests")
    return failed == 0

if __name__ == '__main__':
    success = True

    # Run branch detection tests
    if not test_mips_jalr_branch_detection():
        success = False

    # Run IL lifting tests
    if not test_mips_jalr_il_lifting():
        success = False

    sys.exit(0 if success else 1)
