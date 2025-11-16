#!/usr/bin/env python3
"""
Load binary with detailed logging to see if there are lifter errors
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

# Enable detailed logging
bn.log.log_to_stdout(bn.LogLevel.DebugLog)

print("=" * 70)
print("Loading binary with debug logs")
print("=" * 70)

# Load test binary
bv = bn.load("/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf")
if not bv:
    print("Failed to load binary")
    sys.exit(1)

print("\nWaiting for analysis...")
bv.update_analysis_and_wait()
print("Analysis complete")

# Get function
funcs = bv.get_functions_containing(0x4424)
if funcs:
    func = funcs[0]
    print(f"\nFunction: {func.name}")
    print(f"  Basic blocks: {len(list(func.basic_blocks))}")

    llil = func.low_level_il
    if llil:
        print(f"  LLIL instructions: {len(list(llil.instructions))}")
        print(f"  LLIL basic blocks: {len(list(llil))}")

        print(f"\n  LLIL block details:")
        for bb in llil:
            print(f"    Block {bb.index}: {bb.start}-{bb.end} instructions")

print("\n" + "=" * 70)
