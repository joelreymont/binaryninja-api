#!/usr/bin/env python3
"""
MSP430X LLIL Runtime Validation Test

This script validates the MSP430X LLIL lifting implementation against
the test binary compiled with TI MSP430 GCC 9.3.1.11.
"""

import sys
import os

# Add Binary Ninja Python API to path
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn
from binaryninja import LowLevelILOperation

# Test cases with expected LLIL operations
TEST_CASES = [
    {
        'name': 'RETA',
        'address': 0x4416,
        'bytes': [0x10, 0x01],
        'expected_ops': [LowLevelILOperation.LLIL_RET],
        'description': '20-bit return - should be RET(POP(3))',
    },
    {
        'name': 'PUSHM.A #4, r15',
        'address': 0x4424,
        'bytes': [0x3f, 0x14],
        'expected_ops': [LowLevelILOperation.LLIL_PUSH] * 4,  # 4 PUSH operations
        'description': 'Push 4 registers (r15, r14, r13, r12)',
    },
    {
        'name': 'POPM.A #4, r15',
        'address': 0x442e,
        'bytes': [0x3c, 0x16],
        'expected_ops': [LowLevelILOperation.LLIL_SET_REG] * 4,  # 4 SET_REG with POP
        'description': 'Pop 4 registers (r12, r13, r14, r15)',
    },
    {
        'name': 'RLAM.A #2, r15',
        'address': 0x4434,
        'bytes': [0x4f, 0x06],
        'expected_ops': [LowLevelILOperation.LLIL_SET_REG],
        'expected_expr': LowLevelILOperation.LLIL_LSL,
        'description': 'Logical shift left r15 by 2',
    },
    {
        'name': 'RRAM.A #2, r14',
        'address': 0x4436,
        'bytes': [0x4e, 0x05],
        'expected_ops': [LowLevelILOperation.LLIL_SET_REG],
        'expected_expr': LowLevelILOperation.LLIL_ASR,
        'description': 'Arithmetic shift right r14 by 2',
    },
]

def print_header(text):
    """Print formatted header"""
    print(f"\n{'=' * 70}")
    print(f"  {text}")
    print(f"{'=' * 70}\n")

def print_test_result(name, passed, details=""):
    """Print test result"""
    status = "✓ PASS" if passed else "✗ FAIL"
    print(f"{status:8} {name:30} {details}")

def main():
    print_header("MSP430X LLIL Runtime Validation")

    # Initialize Binary Ninja session
    print("Initializing Binary Ninja headless session...")
    try:
        # For headless operation
        bn.log.log_to_stdout(bn.LogLevel.InfoLog)
        print("✓ Binary Ninja session initialized\n")
    except Exception as e:
        print(f"✗ Failed to initialize Binary Ninja: {e}")
        return 1

    # Load test binary
    test_binary = "/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf"
    if not os.path.exists(test_binary):
        print(f"✗ Test binary not found: {test_binary}")
        return 1

    print(f"Loading test binary: {test_binary}")
    bv = bn.load(test_binary)
    if not bv:
        print(f"✗ Failed to load binary")
        return 1

    print(f"✓ Binary loaded successfully")
    print(f"  Architecture: {bv.arch.name}")
    print(f"  Platform: {bv.platform.name}")
    print(f"  Entry point: {hex(bv.entry_point)}")
    print(f"  Functions: {len(bv.functions)}")

    # Wait for analysis to complete
    print("\nWaiting for analysis to complete...")
    bv.update_analysis_and_wait()
    print("✓ Analysis complete\n")

    # Run tests
    print_header("LLIL Validation Tests")

    total_tests = 0
    passed_tests = 0

    for test in TEST_CASES:
        total_tests += 1
        test_name = f"{test['name']} @ {hex(test['address'])}"

        print(f"\nTest: {test_name}")
        print(f"  Description: {test['description']}")

        # Find function containing this address
        funcs = bv.get_functions_containing(test['address'])
        if not funcs:
            print_test_result(test_name, False, "No function found at address")
            continue

        func = funcs[0]
        llil = func.low_level_il
        if not llil:
            print_test_result(test_name, False, "No LLIL available")
            continue

        # Find LLIL instruction at this address
        llil_instr = None
        for block in llil:
            for instr in block:
                if instr.address == test['address']:
                    llil_instr = instr
                    break
            if llil_instr:
                break

        if not llil_instr:
            print_test_result(test_name, False, "LLIL instruction not found")
            continue

        print(f"  LLIL: {llil_instr}")
        print(f"  Operation: {llil_instr.operation.name}")

        # Validate operation
        expected_op = test['expected_ops'][0]
        if llil_instr.operation == expected_op:
            passed_tests += 1
            print_test_result(test_name, True, f"Correct operation: {expected_op.name}")

            # Additional validation for specific instructions
            if 'expected_expr' in test and hasattr(llil_instr, 'src'):
                if llil_instr.src.operation == test['expected_expr']:
                    print(f"         Expression validation: {test['expected_expr'].name} ✓")
                else:
                    print(f"         Expression mismatch: expected {test['expected_expr'].name}, got {llil_instr.src.operation.name}")
        else:
            print_test_result(test_name, False, f"Expected {expected_op.name}, got {llil_instr.operation.name}")

    # Print summary
    print_header("Test Summary")
    print(f"Total tests: {total_tests}")
    print(f"Passed: {passed_tests}")
    print(f"Failed: {total_tests - passed_tests}")
    print(f"Success rate: {100 * passed_tests / total_tests:.1f}%\n")

    return 0 if passed_tests == total_tests else 1

if __name__ == '__main__':
    sys.exit(main())
