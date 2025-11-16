#!/usr/bin/env python3
"""
Check function boundaries and what's at 0x442e
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

print("=" * 70)
print("Function Boundary Check")
print("=" * 70)

# Load test binary
bv = bn.load("/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf")
if not bv:
    print("Failed to load binary")
    sys.exit(1)

bv.update_analysis_and_wait()

# Check all functions
print("\nAll functions in binary:")
for func in bv.functions:
    print(f"  {func.name:30s} @ {hex(func.start):6s} - {hex(func.highest_address):6s}")

# Check functions containing specific addresses
for addr in [0x4424, 0x442c, 0x442e, 0x4430, 0x4432]:
    funcs = bv.get_functions_containing(addr)
    print(f"\nFunctions containing {hex(addr)}:")
    if funcs:
        for func in funcs:
            print(f"  {func.name} @ {hex(func.start)} - {hex(func.highest_address)}")
    else:
        print(f"  NONE")

# Check if there's a function starting at 0x442e
func_at_442e = bv.get_function_at(0x442e)
print(f"\nFunction starting at 0x442e: {func_at_442e}")

# Get the use_pushm_popm function and examine it
target_func = None
for func in bv.functions:
    if func.start == 0x4424:
        target_func = func
        break

if target_func:
    print(f"\nFunction: {target_func.name}")
    print(f"  Start: {hex(target_func.start)}")
    print(f"  Highest address: {hex(target_func.highest_address)}")
    print(f"  Total size: {hex(target_func.total_bytes)}")

    print(f"\n  Basic blocks:")
    for bb in target_func.basic_blocks:
        print(f"    {hex(bb.start)} - {hex(bb.end)} (length: {bb.length})")
        print(f"      Outgoing edges: {len(bb.outgoing_edges)}")
        for edge in bb.outgoing_edges:
            print(f"        -> {hex(edge.target.start)} ({edge.type})")

    # Check LLIL
    print(f"\n  LLIL basic blocks:")
    llil = target_func.low_level_il
    for bb in llil:
        print(f"    Block {bb.index}: LLIL[{bb.start}:{bb.end}]")
        for i in range(bb.start, bb.end):
            instr = llil[i]
            print(f"      {i:3d} @ {hex(instr.address):6s}: {instr}")

print("\n" + "=" * 70)
