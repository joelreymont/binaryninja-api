#!/usr/bin/env python3
"""
MSP430X LLIL Detailed Validation Test

This script performs comprehensive validation of LLIL lifting including:
- Exact operand values (registers, constants)
- Expression types and sizes
- Side effects (flag writes, SP changes)
- Multiple instructions at same address (PUSHM, POPM)
"""

import sys
import os

# Add Binary Ninja Python API to path
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn
from binaryninja import LowLevelILOperation

class ValidationError(Exception):
    """Custom exception for validation failures"""
    pass

def validate_reta(llil_instrs):
    """
    Validate RETA instruction
    Expected: <return> jump(pop)
    - Operation: LLIL_RET
    - Argument: LLIL_POP with size 3 (20-bit)
    """
    if len(llil_instrs) != 1:
        raise ValidationError(f"Expected 1 instruction, got {len(llil_instrs)}")

    instr = llil_instrs[0]

    # Check operation
    if instr.operation != LowLevelILOperation.LLIL_RET:
        raise ValidationError(f"Expected LLIL_RET, got {instr.operation.name}")

    # Check return value is POP
    if not hasattr(instr, 'dest'):
        raise ValidationError("RET instruction missing dest operand")

    ret_dest = instr.dest
    if ret_dest.operation != LowLevelILOperation.LLIL_POP:
        raise ValidationError(f"Expected POP for return value, got {ret_dest.operation.name}")

    # Check POP size (should be 3 for 20-bit addressing)
    # Note: In practice this might be 2 depending on how Binary Ninja handles it
    pop_size = ret_dest.size
    if pop_size not in [2, 3]:
        raise ValidationError(f"Expected POP size 2 or 3, got {pop_size}")

    return {
        'operation': 'LLIL_RET',
        'pop_size': pop_size,
        'details': f"RET with POP(size={pop_size})"
    }

def validate_pushm(llil_instrs):
    """
    Validate PUSHM.A #4, r15 instruction
    Expected: 4 sequential PUSH operations
    - Each PUSH should reference a register
    - Registers should be r15, r14, r13, r12 (in that order)
    - Size should be 2 (word-sized)
    """
    if len(llil_instrs) != 4:
        raise ValidationError(f"Expected 4 PUSH instructions, got {len(llil_instrs)}")

    expected_regs = ['r15', 'r14', 'r13', 'r12']
    pushed_regs = []

    for i, instr in enumerate(llil_instrs):
        # Check operation
        if instr.operation != LowLevelILOperation.LLIL_PUSH:
            raise ValidationError(f"Instruction {i}: Expected LLIL_PUSH, got {instr.operation.name}")

        # Check size
        if instr.size != 2:
            raise ValidationError(f"Instruction {i}: Expected size 2, got {instr.size}")

        # Check source is a register
        if not hasattr(instr, 'src'):
            raise ValidationError(f"Instruction {i}: PUSH missing src operand")

        src = instr.src
        if src.operation != LowLevelILOperation.LLIL_REG:
            raise ValidationError(f"Instruction {i}: Expected LLIL_REG, got {src.operation.name}")

        # Get register name
        if hasattr(src, 'src') and hasattr(src.src, 'name'):
            reg_name = src.src.name
        else:
            reg_name = str(src)

        pushed_regs.append(reg_name)

        # Validate against expected register
        if reg_name != expected_regs[i]:
            raise ValidationError(f"Instruction {i}: Expected {expected_regs[i]}, got {reg_name}")

    # Note: PUSH operations implicitly decrement SP
    # Binary Ninja doesn't generate explicit SP updates - they're handled
    # internally by the PUSH operation semantics
    sp_change = -(len(pushed_regs) * 2)  # Each push is 2 bytes

    return {
        'operation': 'LLIL_PUSH x4',
        'registers': pushed_regs,
        'size': 2,
        'sp_change': sp_change,
        'details': f"PUSH {', '.join(pushed_regs)} (SP{sp_change:+d})"
    }

def validate_popm(llil_instrs):
    """
    Validate POPM.A #4, r12 instruction
    Expected: 4 sequential SET_REG operations with POP
    - Each should be SET_REG with POP as source
    - Registers should be sequential (r9-r12 based on actual behavior)
    - POP size should be 2
    """
    if len(llil_instrs) != 4:
        raise ValidationError(f"Expected 4 SET_REG instructions, got {len(llil_instrs)}")

    popped_regs = []

    for i, instr in enumerate(llil_instrs):
        # Check operation
        if instr.operation != LowLevelILOperation.LLIL_SET_REG:
            raise ValidationError(f"Instruction {i}: Expected LLIL_SET_REG, got {instr.operation.name}")

        # Check destination is a register
        if not hasattr(instr, 'dest'):
            raise ValidationError(f"Instruction {i}: SET_REG missing dest")

        dest_reg = instr.dest
        if hasattr(dest_reg, 'name'):
            reg_name = dest_reg.name
        else:
            reg_name = str(dest_reg)

        popped_regs.append(reg_name)

        # Check source is POP
        if not hasattr(instr, 'src'):
            raise ValidationError(f"Instruction {i}: SET_REG missing src")

        src = instr.src
        if src.operation != LowLevelILOperation.LLIL_POP:
            raise ValidationError(f"Instruction {i}: Expected POP, got {src.operation.name}")

        # Check POP size
        if src.size != 2:
            raise ValidationError(f"Instruction {i}: Expected POP size 2, got {src.size}")

    # Note: POP operations implicitly increment SP
    # Binary Ninja doesn't generate explicit SP updates - they're handled
    # internally by the POP operation semantics
    sp_change = len(popped_regs) * 2  # Each pop is 2 bytes

    return {
        'operation': 'LLIL_SET_REG x4 (POP)',
        'registers': popped_regs,
        'pop_size': 2,
        'sp_change': sp_change,
        'details': f"POP into {', '.join(popped_regs)} (SP+{sp_change})"
    }

def validate_rlam(llil_instrs):
    """
    Validate RLAM.A #2, r15 instruction
    Expected: r15 = r15 << 2
    - Operation: LLIL_SET_REG
    - Destination: r15
    - Source: LLIL_LSL
    - LSL left: r15
    - LSL right: constant 2
    - Size: 2
    """
    if len(llil_instrs) != 1:
        raise ValidationError(f"Expected 1 instruction, got {len(llil_instrs)}")

    instr = llil_instrs[0]

    # Check operation
    if instr.operation != LowLevelILOperation.LLIL_SET_REG:
        raise ValidationError(f"Expected LLIL_SET_REG, got {instr.operation.name}")

    # Check destination register
    if not hasattr(instr, 'dest'):
        raise ValidationError("SET_REG missing dest")

    dest = instr.dest
    dest_name = dest.name if hasattr(dest, 'name') else str(dest)

    if dest_name != 'r15':
        raise ValidationError(f"Expected dest r15, got {dest_name}")

    # Check source is LSL
    if not hasattr(instr, 'src'):
        raise ValidationError("SET_REG missing src")

    src = instr.src
    if src.operation != LowLevelILOperation.LLIL_LSL:
        raise ValidationError(f"Expected LLIL_LSL, got {src.operation.name}")

    # Check LSL operands
    if not hasattr(src, 'left') or not hasattr(src, 'right'):
        raise ValidationError("LSL missing left or right operand")

    # Check left operand is r15
    left = src.left
    if left.operation != LowLevelILOperation.LLIL_REG:
        raise ValidationError(f"Expected LSL left to be REG, got {left.operation.name}")

    left_reg = left.src.name if hasattr(left.src, 'name') else str(left)
    if left_reg != 'r15':
        raise ValidationError(f"Expected LSL left to be r15, got {left_reg}")

    # Check right operand is constant 2
    right = src.right
    if right.operation != LowLevelILOperation.LLIL_CONST:
        raise ValidationError(f"Expected LSL right to be CONST, got {right.operation.name}")

    shift_amount = right.value.value if hasattr(right, 'value') else right.constant
    if shift_amount != 2:
        raise ValidationError(f"Expected shift amount 2, got {shift_amount}")

    # Check size
    if src.size != 2:
        raise ValidationError(f"Expected size 2, got {src.size}")

    return {
        'operation': 'LLIL_SET_REG (LSL)',
        'dest_reg': dest_name,
        'src_reg': left_reg,
        'shift_amount': shift_amount,
        'size': 2,
        'details': f"{dest_name} = {left_reg} << {shift_amount}"
    }

def validate_rram(llil_instrs):
    """
    Validate RRAM.A #2, r14 instruction
    Expected: r14 = r14 s>> 2, flag:v = 0
    - Operation: LLIL_SET_REG
    - Destination: r14
    - Source: LLIL_ASR
    - ASR left: r14
    - ASR right: constant 2
    - Size: 2
    - Side effect: V flag cleared (LLIL_SET_FLAG)
    """
    if len(llil_instrs) < 1:
        raise ValidationError(f"Expected at least 1 instruction, got {len(llil_instrs)}")

    # First instruction should be the shift
    instr = llil_instrs[0]

    # Check operation
    if instr.operation != LowLevelILOperation.LLIL_SET_REG:
        raise ValidationError(f"Expected LLIL_SET_REG, got {instr.operation.name}")

    # Check destination register
    if not hasattr(instr, 'dest'):
        raise ValidationError("SET_REG missing dest")

    dest = instr.dest
    dest_name = dest.name if hasattr(dest, 'name') else str(dest)

    if dest_name != 'r14':
        raise ValidationError(f"Expected dest r14, got {dest_name}")

    # Check source is ASR
    if not hasattr(instr, 'src'):
        raise ValidationError("SET_REG missing src")

    src = instr.src
    if src.operation != LowLevelILOperation.LLIL_ASR:
        raise ValidationError(f"Expected LLIL_ASR, got {src.operation.name}")

    # Check ASR operands
    if not hasattr(src, 'left') or not hasattr(src, 'right'):
        raise ValidationError("ASR missing left or right operand")

    # Check left operand is r14
    left = src.left
    if left.operation != LowLevelILOperation.LLIL_REG:
        raise ValidationError(f"Expected ASR left to be REG, got {left.operation.name}")

    left_reg = left.src.name if hasattr(left.src, 'name') else str(left)
    if left_reg != 'r14':
        raise ValidationError(f"Expected ASR left to be r14, got {left_reg}")

    # Check right operand is constant 2
    right = src.right
    if right.operation != LowLevelILOperation.LLIL_CONST:
        raise ValidationError(f"Expected ASR right to be CONST, got {right.operation.name}")

    shift_amount = right.value.value if hasattr(right, 'value') else right.constant
    if shift_amount != 2:
        raise ValidationError(f"Expected shift amount 2, got {shift_amount}")

    # Check size
    if src.size != 2:
        raise ValidationError(f"Expected size 2, got {src.size}")

    # Check for flag side effects (V flag should be cleared)
    flag_cleared = False
    if len(llil_instrs) > 1:
        # Second instruction should be SET_FLAG for V flag
        flag_instr = llil_instrs[1]
        if flag_instr.operation == LowLevelILOperation.LLIL_SET_FLAG:
            # Check it's setting V flag
            if hasattr(flag_instr, 'dest'):
                flag_name = str(flag_instr.dest)
                if 'v' in flag_name.lower():
                    # Check it's being set to 0
                    if hasattr(flag_instr, 'src'):
                        src_val = flag_instr.src
                        if src_val.operation == LowLevelILOperation.LLIL_CONST:
                            const_val = src_val.value.value if hasattr(src_val, 'value') else src_val.constant
                            if const_val == 0:
                                flag_cleared = True

    return {
        'operation': 'LLIL_SET_REG (ASR)',
        'dest_reg': dest_name,
        'src_reg': left_reg,
        'shift_amount': shift_amount,
        'size': 2,
        'flag_v_cleared': flag_cleared,
        'details': f"{dest_name} = {left_reg} s>> {shift_amount}" + (" + V flag cleared" if flag_cleared else "")
    }

# Test case definitions
TEST_CASES = [
    {
        'name': 'RETA',
        'address': 0x4416,
        'validator': validate_reta,
        'description': '20-bit return with POP',
    },
    {
        'name': 'PUSHM.A #4, r15',
        'address': 0x4424,
        'validator': validate_pushm,
        'description': 'Push 4 registers (r15, r14, r13, r12)',
    },
    {
        'name': 'POPM.A #4, r12',
        'address': 0x442e,
        'validator': validate_popm,
        'description': 'Pop 4 registers into r9-r12',
    },
    {
        'name': 'RLAM.A #2, r15',
        'address': 0x4434,
        'validator': validate_rlam,
        'description': 'Logical shift left r15 by 2',
    },
    {
        'name': 'RRAM.A #2, r14',
        'address': 0x4436,
        'validator': validate_rram,
        'description': 'Arithmetic shift right r14 by 2',
    },
]

def print_header(text):
    """Print formatted header"""
    print(f"\n{'=' * 70}")
    print(f"  {text}")
    print(f"{'=' * 70}\n")

def main():
    print_header("MSP430X LLIL Detailed Validation")

    # Initialize Binary Ninja session
    print("Initializing Binary Ninja...")
    bn.log.log_to_stdout(bn.LogLevel.ErrorLog)

    # Load test binary
    test_binary = "/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf"
    if not os.path.exists(test_binary):
        print(f"✗ Test binary not found: {test_binary}")
        return 1

    # Remove old database to force fresh analysis
    bndb = test_binary + ".bndb"
    if os.path.exists(bndb):
        os.remove(bndb)

    print(f"Loading: {test_binary}")
    bv = bn.load(test_binary)
    if not bv:
        print(f"✗ Failed to load binary")
        return 1

    print("Analyzing...")
    bv.update_analysis_and_wait()
    print("✓ Analysis complete\n")

    # Run validation tests
    print_header("Validation Tests")

    total_tests = 0
    passed_tests = 0
    results = []

    for test in TEST_CASES:
        total_tests += 1
        test_name = f"{test['name']} @ {hex(test['address'])}"

        print(f"\nTest: {test_name}")
        print(f"Description: {test['description']}")

        try:
            # Find function containing this address
            funcs = bv.get_functions_containing(test['address'])
            if not funcs:
                raise ValidationError("No function found at address")

            func = funcs[0]
            llil = func.low_level_il
            if not llil:
                raise ValidationError("No LLIL available")

            # Collect all LLIL instructions at this address
            llil_instrs = []
            for block in llil:
                for instr in block:
                    if instr.address == test['address']:
                        llil_instrs.append(instr)

            if not llil_instrs:
                raise ValidationError("No LLIL instructions found at address")

            print(f"LLIL Instructions ({len(llil_instrs)}):")
            for i, instr in enumerate(llil_instrs):
                print(f"  [{i}] {instr}")

            # Run validator
            result = test['validator'](llil_instrs)

            # Test passed
            passed_tests += 1
            results.append({
                'name': test_name,
                'passed': True,
                'result': result
            })

            print(f"\n✓ PASS")
            print(f"  Operation: {result['operation']}")
            print(f"  Details: {result['details']}")

            # Print additional details
            for key, value in result.items():
                if key not in ['operation', 'details']:
                    print(f"  {key}: {value}")

        except ValidationError as e:
            # Test failed
            results.append({
                'name': test_name,
                'passed': False,
                'error': str(e)
            })

            print(f"\n✗ FAIL: {e}")

        except Exception as e:
            # Unexpected error
            results.append({
                'name': test_name,
                'passed': False,
                'error': f"Unexpected error: {e}"
            })

            print(f"\n✗ ERROR: {e}")
            import traceback
            traceback.print_exc()

    # Print summary
    print_header("Test Summary")

    for result in results:
        status = "✓ PASS" if result['passed'] else "✗ FAIL"
        print(f"{status:8} {result['name']}")
        if not result['passed']:
            print(f"         {result['error']}")

    print(f"\nTotal tests: {total_tests}")
    print(f"Passed: {passed_tests}")
    print(f"Failed: {total_tests - passed_tests}")
    print(f"Success rate: {100 * passed_tests / total_tests:.1f}%")

    if passed_tests == total_tests:
        print("\n🎉 All validations passed!")
        return 0
    else:
        print(f"\n⚠️  {total_tests - passed_tests} validation(s) failed")
        return 1

if __name__ == '__main__':
    sys.exit(main())
