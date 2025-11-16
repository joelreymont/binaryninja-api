#!/usr/bin/env python3
"""
Debug script for POPM LLIL issue
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

print("=" * 70)
print("POPM LLIL Debug")
print("=" * 70)

# Load test binary
bv = bn.load("/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf")
if not bv:
    print("Failed to load binary")
    sys.exit(1)

bv.update_analysis_and_wait()

# Find function containing POPM at 0x442e
target_addr = 0x442e
funcs = bv.get_functions_containing(target_addr)
if not funcs:
    print(f"No function found containing {hex(target_addr)}")
    sys.exit(1)

func = funcs[0]
print(f"Function: {func.name} @ {hex(func.start)}")
print(f"Function range: {hex(func.start)} - {hex(func.highest_address)}")

# Check disassembly at target address
print(f"\nDisassembly at {hex(target_addr)}:")
arch = bv.arch
for addr in range(target_addr, min(target_addr + 10, func.highest_address + 1), 2):
    data = bv.read(addr, 2)
    if data:
        result = arch.get_instruction_text(data, addr)
        if result:
            tokens, length = result
            text = ''.join(str(t) for t in tokens)
            print(f"  {hex(addr)}: {data.hex()} -> {text} (len={length})")

# Show all LLIL instructions in the function
print(f"\nAll LLIL instructions in function:")
llil = func.low_level_il
if llil:
    idx = 0
    for instr in llil.instructions:
        print(f"  {idx:3d} @ {hex(instr.address):6s}: {instr}")
        idx += 1

# Show LLIL basic blocks
print(f"\nLLIL Basic Blocks:")
for block in llil:
    print(f"  Block {block.index}: {hex(block.start)} - {hex(block.end)}")
    for instr_idx in range(block.start, block.end):
        instr = llil[instr_idx]
        print(f"    {instr_idx:3d} @ {hex(instr.address):6s}: {instr}")

# Check if there's an instruction at exactly 0x442e
print(f"\nSearching for instruction at exactly {hex(target_addr)}:")
found = False
for instr in llil.instructions:
    if instr.address == target_addr:
        print(f"  FOUND: {instr}")
        found = True
        break
if not found:
    print(f"  NOT FOUND")

# Show the disassembly basic blocks too
print(f"\nDisassembly Basic Blocks:")
for block in func.basic_blocks:
    print(f"  Block: {hex(block.start)} - {hex(block.end)}")
    for addr in block:
        data = bv.read(addr, arch.max_instr_length)
        result = arch.get_instruction_text(data, addr)
        if result:
            tokens, _ = result
            text = ''.join(str(t) for t in tokens)
            print(f"    {hex(addr)}: {text}")

print("\n" + "=" * 70)
